// ============================================================
// Oxide Compiler — Recursive Descent Parser
// ============================================================
// Hand-written parser. No parser generators.
// Produces an AST from a token stream.
//
// Grammar precedence (lowest to highest):
//   Assignment  →  Or  →  And  →  Equality  →  Comparison
//   →  Bitwise  →  Shift  →  Addition  →  Multiplication
//   →  Unary  →  Call/Index/Field  →  Primary
// ============================================================

use crate::token::{Token, TokenKind, Span};
use crate::ast::*;

/// Parser error with diagnostic information.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] Parse error: {}", self.span.line, self.span.column, self.message)
    }
}

/// The Oxide recursive descent parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ── Helpers ──

    fn current(&self) -> &Token {
        &self.tokens[self.pos.min(self.tokens.len() - 1)]
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.current().kind
    }

    fn span(&self) -> Span {
        self.current().span.clone()
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        if *self.peek_kind() == kind {
            Ok(self.advance().clone())
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, found {:?} '{}'", kind, self.peek_kind(), self.current().literal),
                span: self.span(),
            })
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn at_end(&self) -> bool {
        self.check(&TokenKind::Eof)
    }

    // ── Top-Level Parsing ──

    /// Parse a complete Oxide source file into a Program.
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();
        while !self.at_end() {
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        let is_pub = self.match_token(TokenKind::Pub);

        match self.peek_kind().clone() {
            TokenKind::Fn => Ok(Item::Function(self.parse_function(is_pub)?)),
            TokenKind::Struct => Ok(Item::Struct(self.parse_struct(is_pub, false)?)),
            TokenKind::Shared => {
                self.advance(); // consume 'shared'
                if self.check(&TokenKind::Struct) {
                    Ok(Item::Struct(self.parse_struct(is_pub, true)?))
                } else {
                    Err(ParseError {
                        message: "Expected 'struct' after 'shared'".into(),
                        span: self.span(),
                    })
                }
            }
            TokenKind::Enum => Ok(Item::Enum(self.parse_enum(is_pub)?)),
            TokenKind::Use => Ok(Item::UseDecl(self.parse_use()?)),
            TokenKind::Mod => Ok(Item::ModDecl(self.parse_mod(is_pub)?)),
            TokenKind::Extern => Ok(Item::ExternFn(self.parse_extern_fn()?)),
            _ => Err(ParseError {
                message: format!("Unexpected token at top level: {:?}", self.peek_kind()),
                span: self.span(),
            }),
        }
    }

    // ── Functions ──

    fn parse_function(&mut self, is_pub: bool) -> Result<FunctionDecl, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Fn)?;
        let name = self.expect(TokenKind::Identifier)?.literal;
        self.expect(TokenKind::LeftParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RightParen)?;

        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;

        Ok(FunctionDecl { name, is_pub, params, return_type, body, span })
    }

    fn parse_param_list(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RightParen) {
            return Ok(params);
        }
        loop {
            let span = self.span();
            let name = self.expect(TokenKind::Identifier)?.literal;
            self.expect(TokenKind::Colon)?;
            let ty = self.parse_type()?;
            params.push(Param { name, ty, span });
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_extern_fn(&mut self) -> Result<ExternFnDecl, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Extern)?;
        let abi = self.expect(TokenKind::StringLiteral)?.literal;
        self.expect(TokenKind::Fn)?;
        let name = self.expect(TokenKind::Identifier)?.literal;
        self.expect(TokenKind::LeftParen)?;
        let params = self.parse_param_list()?;
        self.expect(TokenKind::RightParen)?;

        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect(TokenKind::Semicolon)?;

        Ok(ExternFnDecl { abi, name, params, return_type, span })
    }

    // ── Structs ──

    fn parse_struct(&mut self, is_pub: bool, is_shared: bool) -> Result<StructDecl, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Struct)?;
        let name = self.expect(TokenKind::Identifier)?.literal;
        self.expect(TokenKind::LeftBrace)?;

        let mut fields = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.at_end() {
            let field_pub = self.match_token(TokenKind::Pub);
            let field_span = self.span();
            let field_name = self.expect(TokenKind::Identifier)?.literal;
            self.expect(TokenKind::Colon)?;
            let field_ty = self.parse_type()?;
            // Allow optional trailing comma
            self.match_token(TokenKind::Comma);
            fields.push(FieldDecl { name: field_name, ty: field_ty, is_pub: field_pub, span: field_span });
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(StructDecl { name, is_pub, is_shared, fields, span })
    }

    // ── Enums ──

    fn parse_enum(&mut self, is_pub: bool) -> Result<EnumDecl, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Enum)?;
        let name = self.expect(TokenKind::Identifier)?.literal;
        self.expect(TokenKind::LeftBrace)?;

        let mut variants = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.at_end() {
            let var_span = self.span();
            let var_name = self.expect(TokenKind::Identifier)?.literal;
            let mut fields = Vec::new();
            if self.match_token(TokenKind::LeftParen) {
                while !self.check(&TokenKind::RightParen) {
                    fields.push(self.parse_type()?);
                    if !self.match_token(TokenKind::Comma) { break; }
                }
                self.expect(TokenKind::RightParen)?;
            }
            self.match_token(TokenKind::Comma);
            variants.push(EnumVariant { name: var_name, fields, span: var_span });
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(EnumDecl { name, is_pub, variants, span })
    }

    // ── Use / Mod ──

    /// Consume the current token as a name if it is an Identifier or a keyword.
    /// Keywords are valid in path segments (e.g., `use std::sync::atomic;`).
    fn expect_name(&mut self) -> Result<String, ParseError> {
        let tok = self.current().clone();
        match tok.kind {
            TokenKind::Identifier | TokenKind::Atomic | TokenKind::Shared |
            TokenKind::Region | TokenKind::Unsafe | TokenKind::Drop |
            TokenKind::Mod | TokenKind::Fn | TokenKind::Struct |
            TokenKind::Enum | TokenKind::Match | TokenKind::Pub => {
                self.advance();
                Ok(tok.literal)
            }
            _ => Err(ParseError {
                message: format!("Expected name, found {:?} '{}'", tok.kind, tok.literal),
                span: self.span(),
            }),
        }
    }

    fn parse_use(&mut self) -> Result<UseDecl, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Use)?;
        let mut path = vec![self.expect_name()?];
        while self.match_token(TokenKind::ColonColon) {
            path.push(self.expect_name()?);
        }
        self.expect(TokenKind::Semicolon)?;
        Ok(UseDecl { path, span })
    }

    fn parse_mod(&mut self, is_pub: bool) -> Result<ModDecl, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Mod)?;
        let name = self.expect(TokenKind::Identifier)?.literal;
        self.expect(TokenKind::Semicolon)?;
        Ok(ModDecl { name, is_pub, span })
    }

    // ── Type Expressions ──

    fn parse_type(&mut self) -> Result<TypeExpr, ParseError> {
        let span = self.span();
        match self.peek_kind().clone() {
            TokenKind::Amp => {
                self.advance();
                if self.match_token(TokenKind::Mut) {
                    let inner = self.parse_type()?;
                    Ok(TypeExpr::MutRef(Box::new(inner), span))
                } else {
                    let inner = self.parse_type()?;
                    Ok(TypeExpr::Ref(Box::new(inner), span))
                }
            }
            TokenKind::Star => {
                self.advance();
                if self.match_token(TokenKind::Mut) {
                    let inner = self.parse_type()?;
                    Ok(TypeExpr::MutPtr(Box::new(inner), span))
                } else {
                    let inner = self.parse_type()?;
                    Ok(TypeExpr::Ptr(Box::new(inner), span))
                }
            }
            TokenKind::Atomic => {
                self.advance();
                self.expect(TokenKind::Less)?;
                let inner = self.parse_type()?;
                self.expect(TokenKind::Greater)?;
                Ok(TypeExpr::Atomic(Box::new(inner), span))
            }
            TokenKind::LeftBracket => {
                self.advance();
                let elem = self.parse_type()?;
                self.expect(TokenKind::Semicolon)?;
                let size = self.parse_expression()?;
                self.expect(TokenKind::RightBracket)?;
                Ok(TypeExpr::Array(Box::new(elem), Box::new(size), span))
            }
            TokenKind::Identifier => {
                let mut name = self.advance().literal.clone();
                while self.match_token(TokenKind::ColonColon) {
                    let next = self.expect(TokenKind::Identifier)?.literal;
                    name.push_str("::");
                    name.push_str(&next);
                }
                Ok(TypeExpr::Named(name, span))
            }
            TokenKind::LeftParen => {
                self.advance();
                self.expect(TokenKind::RightParen)?;
                Ok(TypeExpr::Unit(span))
            }
            TokenKind::Pipe => {
                self.advance();
                let mut params = Vec::new();
                if !self.check(&TokenKind::Pipe) {
                    loop {
                        params.push(self.parse_type()?);
                        if !self.match_token(TokenKind::Comma) { break; }
                    }
                }
                self.expect(TokenKind::Pipe)?;
                self.expect(TokenKind::Arrow)?;
                let ret = self.parse_type()?;
                Ok(TypeExpr::Closure(params, Box::new(ret), span))
            }
            TokenKind::PipePipe => {
                self.advance(); // consume ||
                self.expect(TokenKind::Arrow)?;
                let ret = self.parse_type()?;
                Ok(TypeExpr::Closure(Vec::new(), Box::new(ret), span))
            }
            _ => Err(ParseError {
                message: format!("Expected type, found {:?}", self.peek_kind()),
                span,
            }),
        }
    }

    // ── Blocks & Statements ──

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let span = self.span();
        self.expect(TokenKind::LeftBrace)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.at_end() {
            stmts.push(self.parse_statement()?);
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(Block { stmts, span })
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind().clone() {
            TokenKind::Let    => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If     => self.parse_if(),
            TokenKind::While  => self.parse_while(),
            TokenKind::For    => self.parse_for(),
            TokenKind::Loop   => self.parse_loop(),
            TokenKind::Match  => self.parse_match(),
            TokenKind::Region => self.parse_region(),
            TokenKind::Shared => {
                // shared region r { ... }
                if let Some(next) = self.tokens.get(self.pos + 1) {
                    if next.kind == TokenKind::Region {
                        return self.parse_shared_region();
                    }
                }
                // Otherwise treat as expression statement
                self.parse_expr_or_assignment_stmt()
            }
            TokenKind::Unsafe => self.parse_unsafe_block(),
            TokenKind::Break  => {
                let span = self.span();
                self.advance();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Break(span))
            }
            TokenKind::Continue => {
                let span = self.span();
                self.advance();
                self.expect(TokenKind::Semicolon)?;
                Ok(Stmt::Continue(span))
            }
            _ => self.parse_expr_or_assignment_stmt(),
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Let)?;
        let is_mut = self.match_token(TokenKind::Mut);
        let name = self.expect(TokenKind::Identifier)?.literal;

        let ty = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let initializer = if self.match_token(TokenKind::Equal) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Let(LetStmt { name, is_mut, ty, initializer, span }))
    }

    fn parse_return(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Return)?;
        let value = if self.check(&TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Stmt::Return(value, span))
    }

    fn parse_if(&mut self) -> Result<Stmt, ParseError> {
        let if_stmt = self.parse_if_inner()?;
        Ok(Stmt::If(if_stmt))
    }

    fn parse_if_inner(&mut self) -> Result<IfStmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::If)?;
        let condition = self.parse_expression()?;
        let then_block = self.parse_block()?;

        let else_branch = if self.match_token(TokenKind::Else) {
            if self.check(&TokenKind::If) {
                Some(Box::new(ElseBranch::ElseIf(self.parse_if_inner()?)))
            } else {
                Some(Box::new(ElseBranch::Else(self.parse_block()?)))
            }
        } else {
            None
        };

        Ok(IfStmt { condition, then_block, else_branch, span })
    }

    fn parse_while(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Stmt::While(WhileStmt { condition, body, span }))
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::For)?;
        let variable = self.expect(TokenKind::Identifier)?.literal;
        self.expect(TokenKind::In)?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Stmt::For(ForStmt { variable, iterable, body, span }))
    }

    fn parse_loop(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Loop)?;
        let body = self.parse_block()?;
        Ok(Stmt::Loop(body, span))
    }

    fn parse_match(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Match)?;
        let subject = self.parse_expression()?;
        self.expect(TokenKind::LeftBrace)?;

        let mut arms = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.at_end() {
            let arm_span = self.span();
            let pattern = self.parse_pattern()?;
            self.expect(TokenKind::FatArrow)?;
            let body = self.parse_expression()?;
            self.match_token(TokenKind::Comma);
            arms.push(MatchArm { pattern, body, span: arm_span });
        }
        self.expect(TokenKind::RightBrace)?;

        Ok(Stmt::Match(MatchStmt { subject, arms, span }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let span = self.span();
        if self.current().literal == "_" && *self.peek_kind() == TokenKind::Identifier {
            self.advance();
            return Ok(Pattern::Wildcard(span));
        }

        match self.peek_kind().clone() {
            TokenKind::IntLiteral | TokenKind::FloatLiteral |
            TokenKind::StringLiteral | TokenKind::BoolLiteral => {
                let expr = self.parse_primary()?;
                Ok(Pattern::Literal(expr))
            }
            TokenKind::Identifier => {
                let name = self.advance().literal.clone();
                if self.match_token(TokenKind::LeftParen) {
                    let mut sub_patterns = Vec::new();
                    while !self.check(&TokenKind::RightParen) {
                        sub_patterns.push(self.parse_pattern()?);
                        if !self.match_token(TokenKind::Comma) { break; }
                    }
                    self.expect(TokenKind::RightParen)?;
                    Ok(Pattern::Variant(name, sub_patterns, span))
                } else {
                    Ok(Pattern::Ident(name, span))
                }
            }
            _ => Err(ParseError {
                message: format!("Expected pattern, found {:?}", self.peek_kind()),
                span,
            }),
        }
    }

    fn parse_region(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Region)?;
        let name = self.expect(TokenKind::Identifier)?.literal;
        let body = self.parse_block()?;
        Ok(Stmt::Region(RegionBlock { name, is_shared: false, body, span }))
    }

    fn parse_shared_region(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Shared)?;
        self.expect(TokenKind::Region)?;
        let name = self.expect(TokenKind::Identifier)?.literal;
        let body = self.parse_block()?;
        Ok(Stmt::Region(RegionBlock { name, is_shared: true, body, span }))
    }

    fn parse_unsafe_block(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        self.expect(TokenKind::Unsafe)?;
        let body = self.parse_block()?;
        Ok(Stmt::Unsafe(body, span))
    }

    // ── Expression or Assignment ──

    fn parse_expr_or_assignment_stmt(&mut self) -> Result<Stmt, ParseError> {
        let span = self.span();
        let expr = self.parse_expression()?;

        // Check for assignment operators
        let assign_op = match self.peek_kind() {
            TokenKind::Equal          => Some(AssignOp::Assign),
            TokenKind::PlusEqual      => Some(AssignOp::AddAssign),
            TokenKind::MinusEqual     => Some(AssignOp::SubAssign),
            TokenKind::StarEqual      => Some(AssignOp::MulAssign),
            TokenKind::SlashEqual     => Some(AssignOp::DivAssign),
            TokenKind::AmpEqual       => Some(AssignOp::BitAndAssign),
            TokenKind::PipeEqual      => Some(AssignOp::BitOrAssign),
            TokenKind::CaretEqual     => Some(AssignOp::BitXorAssign),
            TokenKind::ShiftLeftEqual => Some(AssignOp::ShlAssign),
            TokenKind::ShiftRightEqual=> Some(AssignOp::ShrAssign),
            _ => None,
        };

        if let Some(op) = assign_op {
            self.advance(); // consume the operator
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Stmt::Assignment(AssignStmt { target: expr, op, value, span }))
        } else {
            self.expect(TokenKind::Semicolon)?;
            Ok(Stmt::ExprStmt(expr))
        }
    }

    // ── Expression Parsing (Pratt-style precedence climbing) ──

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and()?;
        while self.check(&TokenKind::PipePipe) {
            let span = self.span();
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::Or, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_equality()?;
        while self.check(&TokenKind::AmpAmp) {
            let span = self.span();
            self.advance();
            let right = self.parse_equality()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::And, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            let span = self.span();
            match self.peek_kind() {
                TokenKind::EqualEqual => { self.advance(); let r = self.parse_comparison()?; left = Expr::BinaryOp(Box::new(left), BinOp::Eq, Box::new(r), span); }
                TokenKind::BangEqual  => { self.advance(); let r = self.parse_comparison()?; left = Expr::BinaryOp(Box::new(left), BinOp::Neq, Box::new(r), span); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_or()?;
        loop {
            let span = self.span();
            match self.peek_kind() {
                TokenKind::Less         => { self.advance(); let r = self.parse_bitwise_or()?; left = Expr::BinaryOp(Box::new(left), BinOp::Lt, Box::new(r), span); }
                TokenKind::Greater      => { self.advance(); let r = self.parse_bitwise_or()?; left = Expr::BinaryOp(Box::new(left), BinOp::Gt, Box::new(r), span); }
                TokenKind::LessEqual    => { self.advance(); let r = self.parse_bitwise_or()?; left = Expr::BinaryOp(Box::new(left), BinOp::LtEq, Box::new(r), span); }
                TokenKind::GreaterEqual => { self.advance(); let r = self.parse_bitwise_or()?; left = Expr::BinaryOp(Box::new(left), BinOp::GtEq, Box::new(r), span); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_bitwise_or(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_xor()?;
        while self.check(&TokenKind::Pipe) {
            let span = self.span();
            self.advance();
            let right = self.parse_bitwise_xor()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::BitOr, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_bitwise_xor(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_bitwise_and()?;
        while self.check(&TokenKind::Caret) {
            let span = self.span();
            self.advance();
            let right = self.parse_bitwise_and()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::BitXor, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_bitwise_and(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_shift()?;
        while self.check(&TokenKind::Amp) {
            let span = self.span();
            self.advance();
            let right = self.parse_shift()?;
            left = Expr::BinaryOp(Box::new(left), BinOp::BitAnd, Box::new(right), span);
        }
        Ok(left)
    }

    fn parse_shift(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_addition()?;
        loop {
            let span = self.span();
            match self.peek_kind() {
                TokenKind::ShiftLeft  => { self.advance(); let r = self.parse_addition()?; left = Expr::BinaryOp(Box::new(left), BinOp::Shl, Box::new(r), span); }
                TokenKind::ShiftRight => { self.advance(); let r = self.parse_addition()?; left = Expr::BinaryOp(Box::new(left), BinOp::Shr, Box::new(r), span); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_multiplication()?;
        loop {
            let span = self.span();
            match self.peek_kind() {
                TokenKind::Plus  => { self.advance(); let r = self.parse_multiplication()?; left = Expr::BinaryOp(Box::new(left), BinOp::Add, Box::new(r), span); }
                TokenKind::Minus => { self.advance(); let r = self.parse_multiplication()?; left = Expr::BinaryOp(Box::new(left), BinOp::Sub, Box::new(r), span); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let span = self.span();
            match self.peek_kind() {
                TokenKind::Star    => { self.advance(); let r = self.parse_unary()?; left = Expr::BinaryOp(Box::new(left), BinOp::Mul, Box::new(r), span); }
                TokenKind::Slash   => { self.advance(); let r = self.parse_unary()?; left = Expr::BinaryOp(Box::new(left), BinOp::Div, Box::new(r), span); }
                TokenKind::Percent => { self.advance(); let r = self.parse_unary()?; left = Expr::BinaryOp(Box::new(left), BinOp::Mod, Box::new(r), span); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let span = self.span();
        match self.peek_kind().clone() {
            TokenKind::Minus => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(operand), span))
            }
            TokenKind::Bang => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(operand), span))
            }
            TokenKind::Tilde => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::BitNot, Box::new(operand), span))
            }
            TokenKind::Amp => {
                self.advance();
                if self.match_token(TokenKind::Mut) {
                    let operand = self.parse_unary()?;
                    Ok(Expr::UnaryOp(UnaryOp::MutRef, Box::new(operand), span))
                } else {
                    let operand = self.parse_unary()?;
                    Ok(Expr::UnaryOp(UnaryOp::Ref, Box::new(operand), span))
                }
            }
            TokenKind::Star => {
                self.advance();
                let operand = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Deref, Box::new(operand), span))
            }
            _ => self.parse_postfix(),
        }
    }

    // ── Postfix: calls, field access, indexing ──

    fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek_kind().clone() {
                TokenKind::LeftParen => {
                    let span = self.span();
                    self.advance();
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(TokenKind::Comma) { break; }
                        }
                    }
                    self.expect(TokenKind::RightParen)?;
                    expr = Expr::Call(Box::new(expr), args, span);
                }
                TokenKind::Dot => {
                    let span = self.span();
                    self.advance();
                    let field = self.expect(TokenKind::Identifier)?.literal;
                    // Check if it's a method call
                    if self.check(&TokenKind::LeftParen) {
                        self.advance();
                        let mut args = Vec::new();
                        if !self.check(&TokenKind::RightParen) {
                            loop {
                                args.push(self.parse_expression()?);
                                if !self.match_token(TokenKind::Comma) { break; }
                            }
                        }
                        self.expect(TokenKind::RightParen)?;
                        expr = Expr::MethodCall(Box::new(expr), field, args, span);
                    } else {
                        expr = Expr::FieldAccess(Box::new(expr), field, span);
                    }
                }
                TokenKind::LeftBracket => {
                    let span = self.span();
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect(TokenKind::RightBracket)?;
                    expr = Expr::Index(Box::new(expr), Box::new(index), span);
                }
                TokenKind::As => {
                    let span = self.span();
                    self.advance();
                    let target_ty = self.parse_type()?;
                    expr = Expr::Cast(Box::new(expr), target_ty, span);
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    // ── Primary Expressions ──

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        let span = self.span();
        let tok = self.current().clone();

        match tok.kind {
            TokenKind::IntLiteral => {
                self.advance();
                let val = tok.literal.replace("_", "");
                let n = if val.starts_with("0x") || val.starts_with("0X") {
                    i64::from_str_radix(&val[2..], 16).unwrap_or(0)
                } else if val.starts_with("0b") || val.starts_with("0B") {
                    i64::from_str_radix(&val[2..], 2).unwrap_or(0)
                } else if val.starts_with("0o") || val.starts_with("0O") {
                    i64::from_str_radix(&val[2..], 8).unwrap_or(0)
                } else {
                    val.parse::<i64>().unwrap_or(0)
                };
                Ok(Expr::IntLit(n, span))
            }
            TokenKind::FloatLiteral => {
                self.advance();
                let val: f64 = tok.literal.parse().unwrap_or(0.0);
                Ok(Expr::FloatLit(val, span))
            }
            TokenKind::StringLiteral => {
                self.advance();
                Ok(Expr::StringLit(tok.literal.clone(), span))
            }
            TokenKind::CharLiteral => {
                self.advance();
                Ok(Expr::CharLit(tok.literal.chars().next().unwrap_or('\0'), span))
            }
            TokenKind::BoolLiteral => {
                self.advance();
                Ok(Expr::BoolLit(tok.literal == "true", span))
            }
            TokenKind::Relaxed => { self.advance(); Ok(Expr::Ordering(MemoryOrdering::Relaxed, span)) }
            TokenKind::Acquire => { self.advance(); Ok(Expr::Ordering(MemoryOrdering::Acquire, span)) }
            TokenKind::Release => { self.advance(); Ok(Expr::Ordering(MemoryOrdering::Release, span)) }
            TokenKind::AcqRel  => { self.advance(); Ok(Expr::Ordering(MemoryOrdering::AcqRel, span)) }
            TokenKind::SeqCst  => { self.advance(); Ok(Expr::Ordering(MemoryOrdering::SeqCst, span)) }
            
            TokenKind::SizeOf => {
                self.advance();
                self.expect(TokenKind::Less)?;
                let ty = self.parse_type()?;
                self.expect(TokenKind::Greater)?;
                self.expect(TokenKind::LeftParen)?;
                self.expect(TokenKind::RightParen)?;
                Ok(Expr::SizeOf(ty, span))
            }
            TokenKind::AlignOf => {
                self.advance();
                self.expect(TokenKind::Less)?;
                let ty = self.parse_type()?;
                self.expect(TokenKind::Greater)?;
                self.expect(TokenKind::LeftParen)?;
                self.expect(TokenKind::RightParen)?;
                Ok(Expr::AlignOf(ty, span))
            }
            TokenKind::OffsetOf => {
                self.advance();
                self.expect(TokenKind::Less)?;
                let ty = self.parse_type()?;
                self.expect(TokenKind::Greater)?;
                self.expect(TokenKind::LeftParen)?;
                let field = self.expect(TokenKind::Identifier)?.literal;
                self.expect(TokenKind::RightParen)?;
                Ok(Expr::OffsetOf(ty, field, span))
            }
            TokenKind::Identifier => {
                self.advance();
                let name = tok.literal.clone();

                // Check for abort("msg") intrinsic
                if name == "abort" && self.check(&TokenKind::LeftParen) {
                    self.advance(); // (
                    let msg = if self.check(&TokenKind::StringLiteral) {
                        let inner_tok = self.current().clone();
                        self.advance();
                        Some(inner_tok.literal)
                    } else {
                        None
                    };
                    self.expect(TokenKind::RightParen)?;
                    return Ok(Expr::Abort(msg, span));
                }

                // Check for path expression: std::mem::foo
                if self.check(&TokenKind::ColonColon) {
                    let mut path = vec![name];
                    while self.match_token(TokenKind::ColonColon) {
                        path.push(self.expect(TokenKind::Identifier)?.literal);
                    }
                    return Ok(Expr::Path(path, span));
                }

                // Check for struct literal: Point { x: 1, y: 2 }
                if self.check(&TokenKind::LeftBrace) {
                    // Lookahead: is this `Name { field: expr }` or just a block?
                    // Simple heuristic: if next tokens are `Ident :`, it's a struct literal
                    let is_struct_lit = if let (Some(t1), Some(t2)) = (self.tokens.get(self.pos + 1), self.tokens.get(self.pos + 2)) {
                        t1.kind == TokenKind::Identifier && t2.kind == TokenKind::Colon
                    } else {
                        false
                    };

                    if is_struct_lit {
                        self.advance(); // {
                        let mut fields = Vec::new();
                        while !self.check(&TokenKind::RightBrace) {
                            let field_name = self.expect(TokenKind::Identifier)?.literal;
                            self.expect(TokenKind::Colon)?;
                            let value = self.parse_expression()?;
                            fields.push((field_name, value));
                            if !self.match_token(TokenKind::Comma) { break; }
                        }
                        self.expect(TokenKind::RightBrace)?;
                        return Ok(Expr::StructLit(name, fields, span));
                    }
                }

                Ok(Expr::Ident(name, span))
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expr)
            }
            TokenKind::Pipe => self.parse_closure(),
            TokenKind::PipePipe => self.parse_closure(),
            _ => Err(ParseError {
                message: format!("Unexpected token: {:?} '{}'", tok.kind, tok.literal),
                span,
            }),
        }
    }

    fn parse_closure(&mut self) -> Result<Expr, ParseError> {
        let span = self.span();
        let mut args = Vec::new();

        if self.match_token(TokenKind::PipePipe) {
            // Empty closure: || { ... }
        } else {
            self.expect(TokenKind::Pipe)?;
            if !self.check(&TokenKind::Pipe) {
                loop {
                    let arg = self.expect(TokenKind::Identifier)?.literal;
                    args.push(arg);
                    if !self.match_token(TokenKind::Comma) { break; }
                }
            }
            self.expect(TokenKind::Pipe)?;
        }

        let body = self.parse_block()?;
        Ok(Expr::Closure(args, body, span))
    }
}

// ============================================================
// Unit Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(source: &str) -> Program {
        let tokens = Lexer::new(source).tokenize().expect("Lex failed");
        Parser::new(tokens).parse_program().expect("Parse failed")
    }

    #[test]
    fn test_empty_function() {
        let prog = parse("fn main() {}");
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0] {
            Item::Function(f) => {
                assert_eq!(f.name, "main");
                assert!(f.params.is_empty());
                assert!(f.return_type.is_none());
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_function_with_return() {
        let prog = parse("fn add(a: u32, b: u32) -> u32 { return a + b; }");
        match &prog.items[0] {
            Item::Function(f) => {
                assert_eq!(f.name, "add");
                assert_eq!(f.params.len(), 2);
                assert!(f.return_type.is_some());
                assert_eq!(f.body.stmts.len(), 1);
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_struct_decl() {
        let prog = parse("struct Point { pub x: f32, pub y: f32 }");
        match &prog.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Point");
                assert!(!s.is_shared);
                assert_eq!(s.fields.len(), 2);
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_shared_struct() {
        let prog = parse("shared struct Counter { pub count: atomic<u64> }");
        match &prog.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Counter");
                assert!(s.is_shared);
                assert_eq!(s.fields.len(), 1);
                match &s.fields[0].ty {
                    TypeExpr::Atomic(_, _) => {}
                    _ => panic!("Expected atomic type"),
                }
            }
            _ => panic!("Expected struct"),
        }
    }

    #[test]
    fn test_enum_decl() {
        let prog = parse("enum Result { Ok(u32), Err(i32) }");
        match &prog.items[0] {
            Item::Enum(e) => {
                assert_eq!(e.name, "Result");
                assert_eq!(e.variants.len(), 2);
                assert_eq!(e.variants[0].name, "Ok");
                assert_eq!(e.variants[1].name, "Err");
            }
            _ => panic!("Expected enum"),
        }
    }

    #[test]
    fn test_let_statement() {
        let prog = parse("fn main() { let x: u32 = 10; }");
        match &prog.items[0] {
            Item::Function(f) => {
                assert_eq!(f.body.stmts.len(), 1);
                match &f.body.stmts[0] {
                    Stmt::Let(l) => {
                        assert_eq!(l.name, "x");
                        assert!(!l.is_mut);
                        assert!(l.ty.is_some());
                        assert!(l.initializer.is_some());
                    }
                    _ => panic!("Expected let"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_region_block() {
        let prog = parse("fn main() { region r { let x = 10; } }");
        match &prog.items[0] {
            Item::Function(f) => {
                match &f.body.stmts[0] {
                    Stmt::Region(r) => {
                        assert_eq!(r.name, "r");
                        assert!(!r.is_shared);
                        assert_eq!(r.body.stmts.len(), 1);
                    }
                    _ => panic!("Expected region"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_unsafe_block() {
        let prog = parse("fn main() { unsafe { let x = 42; } }");
        match &prog.items[0] {
            Item::Function(f) => {
                match &f.body.stmts[0] {
                    Stmt::Unsafe(block, _) => {
                        assert_eq!(block.stmts.len(), 1);
                    }
                    _ => panic!("Expected unsafe"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_use_declaration() {
        let prog = parse("use std::sync::atomic;");
        match &prog.items[0] {
            Item::UseDecl(u) => {
                assert_eq!(u.path, vec!["std", "sync", "atomic"]);
            }
            _ => panic!("Expected use"),
        }
    }

    #[test]
    fn test_extern_fn() {
        let prog = parse("extern \"c\" fn read(fd: i32, buf: *u8, size: usize) -> isize;");
        match &prog.items[0] {
            Item::ExternFn(e) => {
                assert_eq!(e.abi, "c");
                assert_eq!(e.name, "read");
                assert_eq!(e.params.len(), 3);
            }
            _ => panic!("Expected extern fn"),
        }
    }

    #[test]
    fn test_if_else() {
        let prog = parse("fn main() { if x > 10 { return 1; } else { return 0; } }");
        match &prog.items[0] {
            Item::Function(f) => {
                match &f.body.stmts[0] {
                    Stmt::If(i) => {
                        assert!(i.else_branch.is_some());
                    }
                    _ => panic!("Expected if"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_match_stmt() {
        let prog = parse("fn main() { match res { Ok(val) => val, Err(e) => e } }");
        match &prog.items[0] {
            Item::Function(f) => {
                match &f.body.stmts[0] {
                    Stmt::Match(m) => {
                        assert_eq!(m.arms.len(), 2);
                    }
                    _ => panic!("Expected match"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_method_call() {
        let prog = parse("fn main() { counter.fetch_add(1); }");
        match &prog.items[0] {
            Item::Function(f) => {
                match &f.body.stmts[0] {
                    Stmt::ExprStmt(Expr::MethodCall(_, method, args, _)) => {
                        assert_eq!(method, "fetch_add");
                        assert_eq!(args.len(), 1);
                    }
                    _ => panic!("Expected method call"),
                }
            }
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_binary_precedence() {
        // 1 + 2 * 3 should parse as 1 + (2 * 3)
        let prog = parse("fn main() { let x = 1 + 2 * 3; }");
        match &prog.items[0] {
            Item::Function(f) => {
                match &f.body.stmts[0] {
                    Stmt::Let(l) => {
                        match l.initializer.as_ref().unwrap() {
                            Expr::BinaryOp(_, BinOp::Add, right, _) => {
                                match right.as_ref() {
                                    Expr::BinaryOp(_, BinOp::Mul, _, _) => {}
                                    _ => panic!("Expected Mul on right of Add"),
                                }
                            }
                            _ => panic!("Expected BinaryOp Add"),
                        }
                    }
                    _ => panic!("Expected let"),
                }
            }
            _ => panic!("Expected function"),
        }
    }
}
