// ============================================================
// Oxide Compiler — Abstract Syntax Tree (AST)
// ============================================================
// These types define the in-memory representation of parsed
// Oxide programs. Maps directly to GRAMMAR_OUTLINE_v0.1.md.
//
// Design:
//   - No inheritance. Enum-based tagged unions only.
//   - No implicit allocations. All nodes are stack or Vec-owned.
//   - Spans preserved on every node for diagnostics.
// ============================================================

#![allow(dead_code)]

use crate::token::Span;

// ── Top-Level Program ──

/// A complete Oxide source file.
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// A top-level declaration item.
#[derive(Debug, Clone)]
pub enum Item {
    Function(FunctionDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    UseDecl(UseDecl),
    ModDecl(ModDecl),
    ExternFn(ExternFnDecl),
}

// ── Use / Mod ──

#[derive(Debug, Clone)]
pub struct UseDecl {
    pub path: Vec<String>, // e.g. ["std", "sync", "atomic"]
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ModDecl {
    pub name: String,
    pub is_pub: bool,
    pub span: Span,
}

// ── Functions ──

#[derive(Debug, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub is_pub: bool,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ExternFnDecl {
    pub abi: String,       // e.g. "c"
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

// ── Structs & Enums ──

#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub is_pub: bool,
    pub is_shared: bool,
    pub fields: Vec<FieldDecl>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldDecl {
    pub name: String,
    pub ty: TypeExpr,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub is_pub: bool,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<TypeExpr>, // tuple-style variant payloads
    pub span: Span,
}

// ── Type Expressions ──

#[derive(Debug, Clone)]
pub enum TypeExpr {
    /// Named type:  u32, MyStruct, bool
    Named(String, Span),
    /// Reference:  &T
    Ref(Box<TypeExpr>, Span),
    /// Mutable reference:  &mut T
    MutRef(Box<TypeExpr>, Span),
    /// Raw pointer:  *T
    Ptr(Box<TypeExpr>, Span),
    /// Mutable raw pointer:  *mut T
    MutPtr(Box<TypeExpr>, Span),
    /// Atomic wrapper:  atomic<T>
    Atomic(Box<TypeExpr>, Span),
    /// Array type:  [T; N]
    Array(Box<TypeExpr>, Box<Expr>, Span),
    /// Closure type:  |u32, i32| -> void
    Closure(Vec<TypeExpr>, Box<TypeExpr>, Span),
    /// Unit type: ()
    Unit(Span),
}

// ── Statements ──

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// let x: T = expr;  or  let mut x = expr;
    Let(LetStmt),
    /// expr;
    ExprStmt(Expr),
    /// return expr;
    Return(Option<Expr>, Span),
    /// if / else if / else
    If(IfStmt),
    /// while cond { ... }
    While(WhileStmt),
    /// for item in collection { ... }
    For(ForStmt),
    /// loop { ... }
    Loop(Block, Span),
    /// match expr { arms... }
    Match(MatchStmt),
    /// region name { ... }
    Region(RegionBlock),
    /// unsafe { ... }
    Unsafe(Block, Span),
    /// break;
    Break(Span),
    /// continue;
    Continue(Span),
    /// An assignment:  x = expr; or x += expr;
    Assignment(AssignStmt),
}

#[derive(Debug, Clone)]
pub struct LetStmt {
    pub name: String,
    pub is_mut: bool,
    pub ty: Option<TypeExpr>,
    pub initializer: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub else_branch: Option<Box<ElseBranch>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ElseBranch {
    ElseIf(IfStmt),
    Else(Block),
}

#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ForStmt {
    pub variable: String,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub subject: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    /// Literal value: 42, true, "hello"
    Literal(Expr),
    /// Identifier binding: x
    Ident(String, Span),
    /// Enum variant: Ok(val)
    Variant(String, Vec<Pattern>, Span),
    /// Wildcard: _
    Wildcard(Span),
}

#[derive(Debug, Clone)]
pub struct RegionBlock {
    pub name: String,
    pub is_shared: bool,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AssignStmt {
    pub target: Expr,
    pub op: AssignOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum AssignOp {
    Assign,       // =
    AddAssign,    // +=
    SubAssign,    // -=
    MulAssign,    // *=
    DivAssign,    // /=
    BitAndAssign, // &=
    BitOrAssign,  // |=
    BitXorAssign, // ^=
    ShlAssign,    // <<=
    ShrAssign,    // >>=
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryOrdering {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

// ── Expressions ──

#[derive(Debug, Clone)]
pub enum Expr {
    /// Integer literal
    IntLit(i64, Span),
    /// Float literal
    FloatLit(f64, Span),
    /// String literal
    StringLit(String, Span),
    /// Char literal
    CharLit(char, Span),
    /// Boolean literal
    BoolLit(bool, Span),
    /// Variable reference
    Ident(String, Span),
    /// Binary operation: a + b
    BinaryOp(Box<Expr>, BinOp, Box<Expr>, Span),
    /// Unary operation: !x, -x, &x, &mut x, *x
    UnaryOp(UnaryOp, Box<Expr>, Span),
    /// Function call: foo(a, b)
    Call(Box<Expr>, Vec<Expr>, Span),
    /// Method call: obj.method(args)
    MethodCall(Box<Expr>, String, Vec<Expr>, Span),
    /// Field access: obj.field
    FieldAccess(Box<Expr>, String, Span),
    /// Index: arr[i]
    Index(Box<Expr>, Box<Expr>, Span),
    /// Path expression: std::mem::size_of
    Path(Vec<String>, Span),
    /// Struct literal: Point { x: 1.0, y: 2.0 }
    StructLit(String, Vec<(String, Expr)>, Span),
    /// Block expression (for match arms, etc.)
    Block(Block),
    /// Abort / Panic explicitly: abort("message")
    Abort(Option<String>, Span),
    /// Cast expression: expr as T
    Cast(Box<Expr>, TypeExpr, Span),
    /// Memory ordering for atomics
    Ordering(MemoryOrdering, Span),
    /// Closure expression: |arg1, arg2| { body }
    Closure(Vec<String>, Block, Span),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, LtEq, GtEq,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,      // -
    Not,      // !
    Ref,      // &
    MutRef,   // &mut
    Deref,    // *
    BitNot,   // ~
}
