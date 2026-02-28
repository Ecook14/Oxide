// ============================================================
// Oxide Compiler — Token Definitions
// ============================================================
// Maps directly to GRAMMAR_OUTLINE_v0.1.md lexical structure.
// Every keyword, operator, and literal type is explicitly enumerated.
// No implicit token categories. No regex-based fallback.
// ============================================================

/// Source location for diagnostic reporting.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
    pub offset: usize, // byte offset in source
}

impl Span {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Span { line, column, offset }
    }
}

/// A token is a lexical unit with a kind and a position in the source.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub literal: String, // raw text as written in source
    pub span: Span,
}

/// All possible token kinds in the Oxide language.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Literals ──
    IntLiteral,       // 42, 0xFF, 0b1010
    FloatLiteral,     // 3.14
    StringLiteral,    // "hello"
    CharLiteral,      // 'a'
    BoolLiteral,      // true, false

    // ── Identifiers ──
    Identifier,       // user-defined names

    // ── Core Keywords ──
    Fn,
    Struct,
    Enum,
    Let,
    Mut,
    If,
    Else,
    While,
    For,
    Loop,
    Break,
    Continue,
    Return,
    Match,
    In,
    As,

    // ── Memory & Concurrency Keywords ──
    Region,
    Shared,
    Atomic,
    Unsafe,
    Drop,

    // ── Atomic Ordering Keywords ──
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,

    // ── Module Keywords ──
    Use,
    Pub,
    Mod,

    // ── ABI Keywords ──
    Extern,

    // ── Arithmetic Operators ──
    Plus,             // +
    Minus,            // -
    Star,             // *
    Slash,            // /
    Percent,          // %

    // ── Comparison Operators ──
    EqualEqual,       // ==
    BangEqual,        // !=
    Less,             // <
    Greater,          // >
    LessEqual,        // <=
    GreaterEqual,     // >=

    // ── Logical Operators ──
    AmpAmp,           // &&
    PipePipe,         // ||
    Bang,             // !

    // ── Bitwise Operators ──
    Amp,              // &
    Pipe,             // |
    Caret,            // ^
    ShiftLeft,        // <<
    ShiftRight,       // >>
    Tilde,            // ~

    // ── Assignment Operators ──
    Equal,            // =
    PlusEqual,        // +=
    MinusEqual,       // -=
    StarEqual,        // *=
    SlashEqual,       // /=
    AmpEqual,         // &=
    PipeEqual,        // |=
    CaretEqual,       // ^=
    ShiftLeftEqual,   // <<=
    ShiftRightEqual,  // >>=

    // ── Delimiters ──
    LeftParen,        // (
    RightParen,       // )
    LeftBrace,        // {
    RightBrace,       // }
    LeftBracket,      // [
    RightBracket,     // ]

    // ── Punctuation ──
    Comma,            // ,
    Colon,            // :
    ColonColon,       // ::
    Semicolon,        // ;
    Arrow,            // ->
    FatArrow,         // =>
    Dot,              // .

    // ── Special ──
    Eof,
}

impl TokenKind {
    /// Attempt to match a keyword string to a TokenKind.
    /// Returns None if the string is a user identifier.
    pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
        match s {
            "fn"       => Some(TokenKind::Fn),
            "struct"   => Some(TokenKind::Struct),
            "enum"     => Some(TokenKind::Enum),
            "let"      => Some(TokenKind::Let),
            "mut"      => Some(TokenKind::Mut),
            "if"       => Some(TokenKind::If),
            "else"     => Some(TokenKind::Else),
            "while"    => Some(TokenKind::While),
            "for"      => Some(TokenKind::For),
            "loop"     => Some(TokenKind::Loop),
            "break"    => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "return"   => Some(TokenKind::Return),
            "match"    => Some(TokenKind::Match),
            "in"       => Some(TokenKind::In),
            "as"       => Some(TokenKind::As),
            "region"   => Some(TokenKind::Region),
            "shared"   => Some(TokenKind::Shared),
            "atomic"   => Some(TokenKind::Atomic),
            "unsafe"   => Some(TokenKind::Unsafe),
            "drop"     => Some(TokenKind::Drop),
            "use"      => Some(TokenKind::Use),
            "pub"      => Some(TokenKind::Pub),
            "mod"      => Some(TokenKind::Mod),
            "extern"   => Some(TokenKind::Extern),
            "relaxed"  => Some(TokenKind::Relaxed),
            "acquire"  => Some(TokenKind::Acquire),
            "release"  => Some(TokenKind::Release),
            "acq_rel"  => Some(TokenKind::AcqRel),
            "seq_cst"  => Some(TokenKind::SeqCst),
            "true"     => Some(TokenKind::BoolLiteral),
            "false"    => Some(TokenKind::BoolLiteral),
            _          => None,
        }
    }
}
