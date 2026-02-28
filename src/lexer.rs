// ============================================================
// Oxide Compiler — Lexer (Tokenizer)
// ============================================================
// Hand-written, high-performance lexer. No regex, no external
// dependencies. Deterministic character-by-character scanning.
//
// Design decisions:
//   - Single-pass, zero-allocation for keyword matching.
//   - All errors include source Span for diagnostics.
//   - Strictly maps to token.rs definitions.
// ============================================================

use crate::token::{Token, TokenKind, Span};

/// Lexer error with location information.
#[derive(Debug, Clone)]
pub struct LexerError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for LexerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] Lexer error: {}", self.span.line, self.span.column, self.message)
    }
}

/// The Oxide lexer. Consumes source text and produces a token stream.
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire source into a Vec<Token>.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }

    // ── Core scanning ──

    fn current(&self) -> Option<char> {
        self.source.get(self.pos).copied()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.current()?;
        self.pos += 1;
        if ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn span_here(&self) -> Span {
        Span::new(self.line, self.column, self.pos)
    }

    fn make_token(&self, kind: TokenKind, literal: &str, span: Span) -> Token {
        Token { kind, literal: literal.to_string(), span }
    }

    // ── Whitespace & comments ──

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while let Some(ch) = self.current() {
                if ch.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            // Skip line comments: //
            if self.current() == Some('/') && self.peek() == Some('/') {
                while let Some(ch) = self.current() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }

            // Skip block comments: /* ... */
            if self.current() == Some('/') && self.peek() == Some('*') {
                self.advance(); // /
                self.advance(); // *
                let mut depth = 1;
                while depth > 0 {
                    match self.current() {
                        Some('/') if self.peek() == Some('*') => {
                            self.advance();
                            self.advance();
                            depth += 1;
                        }
                        Some('*') if self.peek() == Some('/') => {
                            self.advance();
                            self.advance();
                            depth -= 1;
                        }
                        Some(_) => { self.advance(); }
                        None => break,
                    }
                }
                continue;
            }

            break;
        }
    }

    // ── Main dispatch ──

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace_and_comments();

        let span = self.span_here();

        let ch = match self.current() {
            Some(c) => c,
            None => return Ok(self.make_token(TokenKind::Eof, "", span)),
        };

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return self.lex_identifier_or_keyword();
        }

        // Number literals
        if ch.is_ascii_digit() {
            return self.lex_number();
        }

        // String literals
        if ch == '"' {
            return self.lex_string();
        }

        // Char literals
        if ch == '\'' {
            return self.lex_char();
        }

        // Operators and punctuation
        self.lex_operator_or_punct()
    }

    // ── Identifiers & Keywords ──

    fn lex_identifier_or_keyword(&mut self) -> Result<Token, LexerError> {
        let span = self.span_here();
        let mut ident = String::new();
        while let Some(ch) = self.current() {
            if ch.is_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let kind = TokenKind::keyword_from_str(&ident)
            .unwrap_or(TokenKind::Identifier);

        Ok(self.make_token(kind, &ident, span))
    }

    // ── Number Literals ──

    fn lex_number(&mut self) -> Result<Token, LexerError> {
        let span = self.span_here();
        let mut num = String::new();
        let mut is_float = false;

        // Handle 0x, 0b, 0o prefixes
        if self.current() == Some('0') {
            match self.peek() {
                Some('x') | Some('X') => {
                    num.push('0');
                    self.advance();
                    num.push('x');
                    self.advance();
                    while let Some(ch) = self.current() {
                        if ch.is_ascii_hexdigit() || ch == '_' {
                            if ch != '_' { num.push(ch); }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    return Ok(self.make_token(TokenKind::IntLiteral, &num, span));
                }
                Some('b') | Some('B') => {
                    num.push('0');
                    self.advance();
                    num.push('b');
                    self.advance();
                    while let Some(ch) = self.current() {
                        if ch == '0' || ch == '1' || ch == '_' {
                            if ch != '_' { num.push(ch); }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    return Ok(self.make_token(TokenKind::IntLiteral, &num, span));
                }
                Some('o') | Some('O') => {
                    num.push('0');
                    self.advance();
                    num.push('o');
                    self.advance();
                    while let Some(ch) = self.current() {
                        if ('0'..='7').contains(&ch) || ch == '_' {
                            if ch != '_' { num.push(ch); }
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    return Ok(self.make_token(TokenKind::IntLiteral, &num, span));
                }
                _ => {}
            }
        }

        // Decimal integer or float
        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() || ch == '_' {
                if ch != '_' { num.push(ch); }
                self.advance();
            } else if ch == '.' && !is_float {
                // Check this isn't a method call (e.g., 42.method())
                if let Some(next) = self.peek() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        num.push('.');
                        self.advance();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let kind = if is_float { TokenKind::FloatLiteral } else { TokenKind::IntLiteral };
        Ok(self.make_token(kind, &num, span))
    }

    // ── String Literals ──

    fn lex_string(&mut self) -> Result<Token, LexerError> {
        let span = self.span_here();
        self.advance(); // opening "
        let mut s = String::new();
        loop {
            match self.current() {
                Some('"') => {
                    self.advance(); // closing "
                    break;
                }
                Some('\\') => {
                    self.advance();
                    match self.current() {
                        Some('n')  => { s.push('\n'); self.advance(); }
                        Some('t')  => { s.push('\t'); self.advance(); }
                        Some('\\') => { s.push('\\'); self.advance(); }
                        Some('"')  => { s.push('"');  self.advance(); }
                        Some('0')  => { s.push('\0'); self.advance(); }
                        Some(c)    => {
                            return Err(LexerError {
                                message: format!("Invalid escape sequence: \\{}", c),
                                span: self.span_here(),
                            });
                        }
                        None => {
                            return Err(LexerError {
                                message: "Unterminated string escape".into(),
                                span,
                            });
                        }
                    }
                }
                Some(ch) => {
                    s.push(ch);
                    self.advance();
                }
                None => {
                    return Err(LexerError {
                        message: "Unterminated string literal".into(),
                        span,
                    });
                }
            }
        }
        Ok(self.make_token(TokenKind::StringLiteral, &s, span))
    }

    // ── Char Literals ──

    fn lex_char(&mut self) -> Result<Token, LexerError> {
        let span = self.span_here();
        self.advance(); // opening '
        let ch = match self.current() {
            Some('\\') => {
                self.advance();
                match self.current() {
                    Some('n')  => { self.advance(); '\n' }
                    Some('t')  => { self.advance(); '\t' }
                    Some('\\') => { self.advance(); '\\' }
                    Some('\'') => { self.advance(); '\'' }
                    Some('0')  => { self.advance(); '\0' }
                    _ => {
                        return Err(LexerError {
                            message: "Invalid char escape".into(),
                            span: self.span_here(),
                        });
                    }
                }
            }
            Some(c) => {
                self.advance();
                c
            }
            None => {
                return Err(LexerError {
                    message: "Unterminated char literal".into(),
                    span,
                });
            }
        };
        if self.current() != Some('\'') {
            return Err(LexerError {
                message: "Expected closing '".into(),
                span: self.span_here(),
            });
        }
        self.advance(); // closing '
        Ok(self.make_token(TokenKind::CharLiteral, &ch.to_string(), span))
    }

    // ── Operators & Punctuation ──

    fn lex_operator_or_punct(&mut self) -> Result<Token, LexerError> {
        let span = self.span_here();
        let ch = self.advance().unwrap();

        let (kind, literal) = match ch {
            '(' => (TokenKind::LeftParen, "("),
            ')' => (TokenKind::RightParen, ")"),
            '{' => (TokenKind::LeftBrace, "{"),
            '}' => (TokenKind::RightBrace, "}"),
            '[' => (TokenKind::LeftBracket, "["),
            ']' => (TokenKind::RightBracket, "]"),
            ',' => (TokenKind::Comma, ","),
            ';' => (TokenKind::Semicolon, ";"),
            '.' => (TokenKind::Dot, "."),
            '~' => (TokenKind::Tilde, "~"),
            ':' => {
                if self.current() == Some(':') {
                    self.advance();
                    (TokenKind::ColonColon, "::")
                } else {
                    (TokenKind::Colon, ":")
                }
            }
            '+' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::PlusEqual, "+=")
                } else {
                    (TokenKind::Plus, "+")
                }
            }
            '-' => {
                if self.current() == Some('>') {
                    self.advance();
                    (TokenKind::Arrow, "->")
                } else if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::MinusEqual, "-=")
                } else {
                    (TokenKind::Minus, "-")
                }
            }
            '*' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::StarEqual, "*=")
                } else {
                    (TokenKind::Star, "*")
                }
            }
            '/' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::SlashEqual, "/=")
                } else {
                    (TokenKind::Slash, "/")
                }
            }
            '%' => (TokenKind::Percent, "%"),
            '=' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::EqualEqual, "==")
                } else if self.current() == Some('>') {
                    self.advance();
                    (TokenKind::FatArrow, "=>")
                } else {
                    (TokenKind::Equal, "=")
                }
            }
            '!' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::BangEqual, "!=")
                } else {
                    (TokenKind::Bang, "!")
                }
            }
            '<' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::LessEqual, "<=")
                } else if self.current() == Some('<') {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        (TokenKind::ShiftLeftEqual, "<<=")
                    } else {
                        (TokenKind::ShiftLeft, "<<")
                    }
                } else {
                    (TokenKind::Less, "<")
                }
            }
            '>' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::GreaterEqual, ">=")
                } else if self.current() == Some('>') {
                    self.advance();
                    if self.current() == Some('=') {
                        self.advance();
                        (TokenKind::ShiftRightEqual, ">>=")
                    } else {
                        (TokenKind::ShiftRight, ">>")
                    }
                } else {
                    (TokenKind::Greater, ">")
                }
            }
            '&' => {
                if self.current() == Some('&') {
                    self.advance();
                    (TokenKind::AmpAmp, "&&")
                } else if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::AmpEqual, "&=")
                } else {
                    (TokenKind::Amp, "&")
                }
            }
            '|' => {
                if self.current() == Some('|') {
                    self.advance();
                    (TokenKind::PipePipe, "||")
                } else if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::PipeEqual, "|=")
                } else {
                    (TokenKind::Pipe, "|")
                }
            }
            '^' => {
                if self.current() == Some('=') {
                    self.advance();
                    (TokenKind::CaretEqual, "^=")
                } else {
                    (TokenKind::Caret, "^")
                }
            }
            _ => {
                return Err(LexerError {
                    message: format!("Unexpected character: '{}'", ch),
                    span,
                });
            }
        };

        Ok(self.make_token(kind, literal, span))
    }
}

// ============================================================
// Unit Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(source: &str) -> Vec<Token> {
        Lexer::new(source).tokenize().expect("Lexer failed")
    }

    fn kinds(source: &str) -> Vec<TokenKind> {
        lex(source).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_keywords() {
        assert_eq!(
            kinds("fn let mut struct enum region shared unsafe"),
            vec![
                TokenKind::Fn, TokenKind::Let, TokenKind::Mut,
                TokenKind::Struct, TokenKind::Enum, TokenKind::Region,
                TokenKind::Shared, TokenKind::Unsafe, TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let tokens = lex("my_var x123 _temp");
        assert_eq!(tokens[0].kind, TokenKind::Identifier);
        assert_eq!(tokens[0].literal, "my_var");
        assert_eq!(tokens[1].literal, "x123");
        assert_eq!(tokens[2].literal, "_temp");
    }

    #[test]
    fn test_integer_literals() {
        assert_eq!(kinds("42 0xFF 0b1010 0o77"), vec![
            TokenKind::IntLiteral, TokenKind::IntLiteral,
            TokenKind::IntLiteral, TokenKind::IntLiteral,
            TokenKind::Eof,
        ]);
    }

    #[test]
    fn test_float_literal() {
        let tokens = lex("3.14");
        assert_eq!(tokens[0].kind, TokenKind::FloatLiteral);
        assert_eq!(tokens[0].literal, "3.14");
    }

    #[test]
    fn test_string_literal() {
        let tokens = lex("\"hello\\nworld\"");
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].literal, "hello\nworld");
    }

    #[test]
    fn test_char_literal() {
        let tokens = lex("'a' '\\n'");
        assert_eq!(tokens[0].kind, TokenKind::CharLiteral);
        assert_eq!(tokens[0].literal, "a");
        assert_eq!(tokens[1].kind, TokenKind::CharLiteral);
        assert_eq!(tokens[1].literal, "\n");
    }

    #[test]
    fn test_operators() {
        assert_eq!(
            kinds("+ - * / % == != < > <= >= && || ! -> =>"),
            vec![
                TokenKind::Plus, TokenKind::Minus, TokenKind::Star,
                TokenKind::Slash, TokenKind::Percent,
                TokenKind::EqualEqual, TokenKind::BangEqual,
                TokenKind::Less, TokenKind::Greater,
                TokenKind::LessEqual, TokenKind::GreaterEqual,
                TokenKind::AmpAmp, TokenKind::PipePipe, TokenKind::Bang,
                TokenKind::Arrow, TokenKind::FatArrow,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_delimiters_and_punctuation() {
        assert_eq!(
            kinds("( ) { } [ ] , : :: ; ."),
            vec![
                TokenKind::LeftParen, TokenKind::RightParen,
                TokenKind::LeftBrace, TokenKind::RightBrace,
                TokenKind::LeftBracket, TokenKind::RightBracket,
                TokenKind::Comma, TokenKind::Colon, TokenKind::ColonColon,
                TokenKind::Semicolon, TokenKind::Dot,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_comments_skipped() {
        assert_eq!(
            kinds("fn // this is a comment\nlet"),
            vec![TokenKind::Fn, TokenKind::Let, TokenKind::Eof]
        );
        assert_eq!(
            kinds("fn /* block */ let"),
            vec![TokenKind::Fn, TokenKind::Let, TokenKind::Eof]
        );
    }

    #[test]
    fn test_full_function() {
        let source = "fn add(a: u32, b: u32) -> u32 { return a + b; }";
        let tokens = lex(source);
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Identifier); // add
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        // ... the parser will validate the full structure
        assert!(tokens.last().unwrap().kind == TokenKind::Eof);
    }

    #[test]
    fn test_region_block() {
        let source = "region r { let x = 10; }";
        assert_eq!(
            kinds(source),
            vec![
                TokenKind::Region, TokenKind::Identifier,
                TokenKind::LeftBrace, TokenKind::Let,
                TokenKind::Identifier, TokenKind::Equal,
                TokenKind::IntLiteral, TokenKind::Semicolon,
                TokenKind::RightBrace, TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_shared_struct() {
        let source = "shared struct Counter { count: atomic }";
        assert_eq!(
            kinds(source),
            vec![
                TokenKind::Shared, TokenKind::Struct, TokenKind::Identifier,
                TokenKind::LeftBrace, TokenKind::Identifier, TokenKind::Colon,
                TokenKind::Atomic, TokenKind::RightBrace, TokenKind::Eof,
            ]
        );
    }
}
