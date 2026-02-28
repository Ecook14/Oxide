// ============================================================
// Oxide Compiler — Semantic Analyzer
// ============================================================
// Performs semantic validation on the AST:
//   1. Type checking & resolution
//   2. Ownership tracking (single-owner, move semantics)
//   3. Borrow validation (lexical, no lifetime annotations)
//   4. Region escape analysis
//   5. Shared struct composition rules
//
// This directly implements MEMORY_MODEL_v0.1.md rules.
// ============================================================

#![allow(dead_code)]

use crate::ast::*;
use crate::token::Span;

// ── Semantic Errors ──

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub code: ErrorCode,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCode {
    // Ownership errors
    UseAfterMove,           // E_USE_AFTER_MOVE
    MutAliasViolation,      // E_MUT_ALIAS_VIOLATION
    BorrowOutlivesOwner,    // E_BORROW_OUTLIVES_OWNER

    // Region errors
    RegionEscape,           // Region Escape Violation

    // Shared struct errors
    SharedFieldNotAtomic,   // Non-atomic mutable field in shared struct

    // Type errors
    TypeMismatch,
    UndefinedVariable,
    UndefinedType,
    UndefinedFunction,

    // General
    DuplicateDefinition,
    InvalidUnsafe,
}

impl std::fmt::Display for SemanticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] {:?}: {}", self.span.line, self.span.column, self.code, self.message)
    }
}

// ── Type Representation ──

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitives
    U8, U16, U32, U64, USize,
    I8, I16, I32, I64, ISize,
    F32, F64,
    Bool,
    Char,
    Void,

    // Compound
    Struct(String),
    Enum(String),
    Ref(Box<Type>),
    MutRef(Box<Type>),
    Ptr(Box<Type>),
    MutPtr(Box<Type>),
    Atomic(Box<Type>),
    Array(Box<Type>, usize),
    Ordering, // For atomic memory orderings (relaxed, seq_cst, etc.)
    Closure(Vec<Type>, Box<Type>), // Closure(Params, Return)


    // Unknown (for inference)
    Inferred,
}

impl Type {
    pub fn from_name(name: &str) -> Option<Type> {
        match name {
            "u8"    => Some(Type::U8),
            "u16"   => Some(Type::U16),
            "u32"   => Some(Type::U32),
            "u64"   => Some(Type::U64),
            "usize" => Some(Type::USize),
            "i8"    => Some(Type::I8),
            "i16"   => Some(Type::I16),
            "i32"   => Some(Type::I32),
            "i64"   => Some(Type::I64),
            "isize" => Some(Type::ISize),
            "f32"   => Some(Type::F32),
            "f64"   => Some(Type::F64),
            "bool"  => Some(Type::Bool),
            "char"  => Some(Type::Char),
            _       => None,
        }
    }

    pub fn is_integer(&self) -> bool {
        matches!(self,
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::USize |
            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::ISize
        )
    }

    pub fn is_atomic_compatible(&self) -> bool {
        matches!(self,
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::USize |
            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::ISize |
            Type::Bool | Type::Ptr(_)
        )
    }
}

// ── Ownership State ──

#[derive(Debug, Clone, PartialEq)]
pub enum OwnershipState {
    Owned,
    Moved,
    ImmutBorrowed(usize),  // count of active immutable borrows
    MutBorrowed,
}

// ── Variable Info ──

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub ty: Type,
    pub is_mut: bool,
    pub ownership: OwnershipState,
    pub region: Option<String>,  // None = stack/heap, Some(r) = allocated in region r
    pub scope_depth: usize,
    pub def_span: Span,
}

// ── Struct Info ──

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub is_shared: bool,
    pub fields: Vec<(String, Type, bool)>, // (name, type, is_pub)
}

// ── Function Info ──

#[derive(Debug, Clone)]
pub struct FuncInfo {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
}

// ── Analyzer ──

pub struct Analyzer {
    // Symbol tables
    variables: Vec<VarInfo>,
    structs: Vec<StructInfo>,
    functions: Vec<FuncInfo>,

    // State
    scope_depth: usize,
    current_region: Option<String>,
    in_unsafe: bool,

    // Output
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<String>,

    // Type checking context
    expected_return_type: Option<Type>,
}

impl Analyzer {
    pub fn new() -> Self {
        Analyzer {
            variables: Vec::new(),
            structs: Vec::new(),
            functions: Vec::new(),
            scope_depth: 0,
            current_region: None,
            in_unsafe: false,
            errors: Vec::new(),
            warnings: Vec::new(),
            expected_return_type: None,
        }
    }

    // ── Public Entry Point ──

    pub fn analyze(&mut self, program: &Program) -> bool {
        // First pass: collect type and function declarations
        for item in &program.items {
            match item {
                Item::Struct(s)   => self.register_struct(s),
                Item::Function(f) => self.register_function(f),
                Item::Enum(e)     => self.register_enum(e),
                Item::ExternFn(e) => self.register_extern_function(e),
                _ => {}
            }
        }

        // Second pass: validate function bodies
        for item in &program.items {
            if let Item::Function(f) = item {
                self.analyze_function(f);
            }
        }

        self.errors.is_empty()
    }

    // ── Registration (First Pass) ──

    fn register_struct(&mut self, s: &StructDecl) {
        // Check for duplicate
        if self.structs.iter().any(|existing| existing.name == s.name) {
            self.emit(ErrorCode::DuplicateDefinition,
                format!("Struct '{}' already defined", s.name), &s.span);
            return;
        }

        let mut fields = Vec::new();
        for field in &s.fields {
            let ty = self.resolve_type_expr(&field.ty);

            // MEMORY_MODEL Rule: shared structs must contain only atomics or immutable fields
            if s.is_shared {
                match &ty {
                    Type::Atomic(_) => {} // OK
                    _ => {
                        // Field is allowed only if it is not mutable
                        // In Oxide, struct fields are immutable by default unless &mut
                        // But for shared structs, any non-atomic field must be deeply immutable
                        if !self.is_deeply_immutable(&ty) {
                            self.emit(ErrorCode::SharedFieldNotAtomic,
                                format!("Shared struct '{}' field '{}' must be atomic<T> or deeply immutable",
                                    s.name, field.name),
                                &field.span);
                        }
                    }
                }
            }

            fields.push((field.name.clone(), ty, field.is_pub));
        }

        self.structs.push(StructInfo {
            name: s.name.clone(),
            is_shared: s.is_shared,
            fields,
        });
    }

    fn register_enum(&mut self, e: &EnumDecl) {
        // Register as a named type for lookup
        if self.structs.iter().any(|s| s.name == e.name) {
            self.emit(ErrorCode::DuplicateDefinition,
                format!("Type '{}' already defined", e.name), &e.span);
        }
    }

    fn register_function(&mut self, f: &FunctionDecl) {
        if self.functions.iter().any(|existing| existing.name == f.name) {
            self.emit(ErrorCode::DuplicateDefinition,
                format!("Function '{}' already defined", f.name), &f.span);
            return;
        }

        let params: Vec<(String, Type)> = f.params.iter()
            .map(|p| (p.name.clone(), self.resolve_type_expr(&p.ty)))
            .collect();

        let return_type = f.return_type.as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or(Type::Void);

        self.functions.push(FuncInfo {
            name: f.name.clone(),
            params,
            return_type,
        });
    }

    fn register_extern_function(&mut self, e: &ExternFnDecl) {
        if self.functions.iter().any(|existing| existing.name == e.name) {
            self.emit(ErrorCode::DuplicateDefinition,
                format!("Function '{}' already defined", e.name), &e.span);
            return;
        }

        let params: Vec<(String, Type)> = e.params.iter()
            .map(|p| (p.name.clone(), self.resolve_type_expr(&p.ty)))
            .collect();

        let return_type = e.return_type.as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or(Type::Void);

        self.functions.push(FuncInfo {
            name: e.name.clone(),
            params,
            return_type,
        });
    }

    // ── Function Body Analysis (Second Pass) ──

    fn analyze_function(&mut self, f: &FunctionDecl) {
        self.enter_scope();

        // Register parameters as local variables
        for param in &f.params {
            let ty = self.resolve_type_expr(&param.ty);
            self.define_var(&param.name, ty, false, &param.span);
        }

        let ret_ty = f.return_type.as_ref()
            .map(|t| self.resolve_type_expr(t))
            .unwrap_or(Type::Void);
            
        let prev_ret = self.expected_return_type.clone();
        self.expected_return_type = Some(ret_ty);

        self.analyze_block(&f.body);
        
        self.expected_return_type = prev_ret;
        self.exit_scope();
    }

    fn analyze_block(&mut self, block: &Block) {
        self.enter_scope();
        for stmt in &block.stmts {
            self.analyze_stmt(stmt);
        }
        self.exit_scope();
    }

    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => self.analyze_let(let_stmt),
            Stmt::ExprStmt(expr) => { self.analyze_expr(expr); }
            Stmt::Return(expr, span) => {
                let actual_ty = if let Some(e) = expr {
                    self.check_region_escape_expr(e);
                    self.analyze_expr(e)
                } else {
                    Type::Void
                };
                
                if let Some(expected_ty) = self.expected_return_type.clone() {
                    if let Some(e) = expr {
                         if !self.check_types_match(&expected_ty, e) {
                             self.emit(ErrorCode::TypeMismatch,
                                format!("Return type mismatch: expected {:?}, got {:?}", expected_ty, actual_ty), span);
                         }
                    } else if expected_ty != Type::Void {
                         self.emit(ErrorCode::TypeMismatch,
                            format!("Return type mismatch: expected {:?}, got Void", expected_ty), span);
                    }
                }
            }
            Stmt::If(if_stmt) => self.analyze_if(if_stmt),
            Stmt::While(while_stmt) => {
                let cond_ty = self.analyze_expr(&while_stmt.condition);
                if cond_ty != Type::Bool && cond_ty != Type::Inferred {
                    self.emit(ErrorCode::TypeMismatch, "while condition must be a boolean".into(), &while_stmt.span);
                }
                self.analyze_block(&while_stmt.body);
            }
            Stmt::For(for_stmt) => {
                self.analyze_expr(&for_stmt.iterable);
                self.enter_scope();
                self.define_var(&for_stmt.variable, Type::Inferred, false, &for_stmt.span);
                self.analyze_block(&for_stmt.body);
                self.exit_scope();
            }
            Stmt::Loop(block, _) => self.analyze_block(block),
            Stmt::Match(match_stmt) => {
                self.analyze_expr(&match_stmt.subject);
                for arm in &match_stmt.arms {
                    self.analyze_expr(&arm.body);
                }
            }
            Stmt::Region(region_block) => self.analyze_region(region_block),
            Stmt::Unsafe(block, _) => {
                let was_unsafe = self.in_unsafe;
                self.in_unsafe = true;
                self.analyze_block(block);
                self.in_unsafe = was_unsafe;
            }
            Stmt::Assignment(assign) => {
                let _target_ty = self.analyze_expr(&assign.target);
                let val_ty = self.analyze_expr(&assign.value);
                
                if _target_ty != Type::Inferred && !self.check_types_match(&_target_ty, &assign.value) {
                    self.emit(ErrorCode::TypeMismatch,
                        format!("Cannot assign type {:?} to {:?}", val_ty, _target_ty), &assign.span);
                }
                
                // Track region escape for assignments
                if let Expr::Ident(target_name, _) = &assign.target {
                    if let Some(target_var) = self.lookup_var(target_name).map(|v| v.clone()) {
                        let sources = self.find_source_variables(&assign.value);
                        for src_name in sources {
                            if let Some(src_var) = self.lookup_var(&src_name) {
                                if let Some(ref r) = src_var.region {
                                    if target_var.region != Some(r.clone()) && target_var.scope_depth < src_var.scope_depth {
                                        if !self.in_unsafe {
                                            self.emit(ErrorCode::RegionEscape,
                                                format!("Value '{}' from region '{}' escapes via assignment to '{}'",
                                                    src_name, r, target_name), &assign.span);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                // Check target is mutable
                if let Expr::Ident(name, span) = &assign.target {
                    let var_info = self.lookup_var(name)
                        .map(|v| (v.is_mut, v.ownership.clone()));
                    if let Some((is_mut, ownership)) = var_info {
                        if !is_mut {
                            self.emit(ErrorCode::TypeMismatch,
                                format!("Cannot assign to immutable variable '{}'", name), span);
                        }
                        if ownership == OwnershipState::Moved {
                            self.emit(ErrorCode::UseAfterMove,
                                format!("Cannot assign to moved variable '{}'", name), span);
                        }
                    }
                }
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
        }
    }

    fn analyze_let(&mut self, let_stmt: &LetStmt) {
        let mut ty = if let Some(type_expr) = &let_stmt.ty {
            self.resolve_type_expr(type_expr)
        } else {
            Type::Inferred
        };

        if let Some(init) = &let_stmt.initializer {
            let init_ty = self.analyze_expr(init);
            if ty == Type::Inferred {
                ty = init_ty;
            } else if !self.check_types_match(&ty, init) && init_ty != Type::Inferred {
                self.emit(ErrorCode::TypeMismatch,
                    format!("Variable '{}' type mismatch: expected {:?}, got {:?}", let_stmt.name, ty, init_ty), &let_stmt.span);
            }
        }

        self.define_var(&let_stmt.name, ty, let_stmt.is_mut, &let_stmt.span);
    }

    fn analyze_if(&mut self, if_stmt: &IfStmt) {
        let cond_ty = self.analyze_expr(&if_stmt.condition);
        if cond_ty != Type::Bool && cond_ty != Type::Inferred {
            self.emit(ErrorCode::TypeMismatch, "if condition must be a boolean".into(), &if_stmt.span);
        }
        self.analyze_block(&if_stmt.then_block);
        if let Some(else_branch) = &if_stmt.else_branch {
            match else_branch.as_ref() {
                ElseBranch::ElseIf(nested) => self.analyze_if(nested),
                ElseBranch::Else(block) => self.analyze_block(block),
            }
        }
    }

    fn analyze_region(&mut self, region: &RegionBlock) {
        let prev_region = self.current_region.clone();
        self.current_region = Some(region.name.clone());
        self.analyze_block(&region.body);

        // MEMORY_MODEL: Region Exit Guarantee - all allocations freed here
        // Check no variable escapes this region
        self.check_region_exit(&region.name, &region.span);

        self.current_region = prev_region;
    }

    // ── Expression Analysis ──

    fn analyze_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::IntLit(_, _) => Type::I64, // Inference simplifies to i64 by default
            Expr::FloatLit(_, _) => Type::F64,
            Expr::BoolLit(_, _) => Type::Bool,
            Expr::CharLit(_, _) => Type::Char,
            Expr::StringLit(_, _) => Type::Ref(Box::new(Type::U8)),
            
            Expr::Ident(name, span) => {
                let is_moved = self.lookup_var(name)
                    .map(|v| v.ownership == OwnershipState::Moved)
                    .unwrap_or(false);
                if is_moved {
                    self.emit(ErrorCode::UseAfterMove,
                        format!("Use of moved value '{}'", name), span);
                }
                
                if let Some(v) = self.lookup_var(name) {
                    v.ty.clone()
                } else {
                    // Could be a function reference
                    if self.functions.iter().any(|f| f.name == *name) {
                        Type::Inferred // Functional types not yet fully modeled
                    } else {
                        self.emit(ErrorCode::UndefinedVariable, format!("Undefined variable '{}'", name), span);
                        Type::Inferred
                    }
                }
            }
            Expr::BinaryOp(left, op, right, span) => {
                let l_ty = self.analyze_expr(left);
                let r_ty = self.analyze_expr(right);
                
                if l_ty != Type::Inferred && r_ty != Type::Inferred {
                    if !self.check_types_match(&l_ty, right) && !self.check_types_match(&r_ty, left) {
                        self.emit(ErrorCode::TypeMismatch, format!("Type mismatch in binary op: {:?} vs {:?}", l_ty, r_ty), span);
                    }
                }
                
                match op {
                    BinOp::Eq | BinOp::Neq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => Type::Bool,
                    _ => l_ty,
                }
            }
            Expr::UnaryOp(op, operand, span) => {
                let inner_ty = self.analyze_expr(operand);
                match op {
                    UnaryOp::MutRef => {
                        if let Expr::Ident(name, _) = operand.as_ref() {
                            self.check_mutable_borrow(name, span);
                        }
                        Type::MutRef(Box::new(inner_ty))
                    }
                    UnaryOp::Ref => {
                        if let Expr::Ident(name, _) = operand.as_ref() {
                            self.check_immutable_borrow(name, span);
                        }
                        Type::Ref(Box::new(inner_ty))
                    }
                    UnaryOp::Deref => {
                        if !self.in_unsafe { /* Need strict rules applied here in future */ }
                        match inner_ty {
                            Type::Ref(t) | Type::MutRef(t) | Type::Ptr(t) | Type::MutPtr(t) => *t,
                            Type::Inferred => Type::Inferred,
                            _ => {
                                self.emit(ErrorCode::TypeMismatch, "Cannot dereference non-pointer/reference type".into(), span);
                                Type::Inferred
                            }
                        }
                    }
                    UnaryOp::Not => Type::Bool,
                    _ => inner_ty,
                }
            }
            Expr::Call(callee, args, span) => {
                let callee_ty = self.analyze_expr(callee);
                let mut arg_tys = Vec::new();
                for arg in args {
                    arg_tys.push(self.analyze_expr(arg));
                }
                
                // Handle closure calls
                if let Type::Closure(params, ret) = &callee_ty {
                    if params.len() != args.len() {
                        self.emit(ErrorCode::TypeMismatch, format!("Closure expects {} arguments, got {}", params.len(), args.len()), span);
                    } else {
                        for (i, (expected, actual)) in params.iter().zip(arg_tys.iter()).enumerate() {
                            if *actual != Type::Inferred && *expected != Type::Inferred && *actual != *expected {
                                self.emit(ErrorCode::TypeMismatch, format!("Argument {} expects type {:?}, got {:?}", i, expected, actual), span);
                            }
                        }
                    }
                    return (**ret).clone();
                }

                // Fallback: Handle direct function calls by name (Phase 1 legacy/simplicity)
                if let Expr::Ident(name, _) = callee.as_ref() {
                    if let Some(f) = self.functions.iter().find(|f| f.name == *name).cloned() {
                        if f.params.len() != args.len() {
                            self.emit(ErrorCode::TypeMismatch, format!("Function '{}' expects {} arguments, got {}", name, f.params.len(), args.len()), span);
                        } else {
                            for (i, (param_ty, actual_ty)) in f.params.iter().map(|p| &p.1).zip(arg_tys.iter()).enumerate() {
                                if *actual_ty != Type::Inferred && *param_ty != Type::Inferred && *actual_ty != *param_ty {
                                    self.emit(ErrorCode::TypeMismatch, format!("Argument {} expects type {:?}, got {:?}", i, param_ty, actual_ty), span);
                                }
                            }
                        }
                        return f.return_type;
                    }
                    
                    // Also check if 'name' is a local variable of type Closure
                    if let Some(var_ty) = self.lookup_var(name).map(|v| v.ty.clone()) {
                        if let Type::Closure(params, ret) = var_ty {
                            // Validate closure call via variable
                            if params.len() != args.len() {
                                self.emit(ErrorCode::TypeMismatch, format!("Closure '{}' expects {} arguments, got {}", name, params.len(), args.len()), span);
                            } else {
                                for (i, (expected, actual)) in params.iter().zip(arg_tys.iter()).enumerate() {
                                    if *actual != Type::Inferred && *expected != Type::Inferred && *actual != *expected {
                                        self.emit(ErrorCode::TypeMismatch, format!("Argument {} expects type {:?}, got {:?}", i, expected, actual), span);
                                    }
                                }
                            }
                            return (*ret).clone();
                        }
                    }
                }
                
                Type::Inferred
            }
            Expr::MethodCall(receiver, method, args, span) => {
                let recv_ty = self.analyze_expr(receiver);
                let mut arg_tys = Vec::new();
                for arg in args {
                    arg_tys.push(self.analyze_expr(arg));
                }

                if let Type::Atomic(inner) = recv_ty {
                    match method.as_str() {
                        "load" => {
                            if args.len() != 1 || arg_tys[0] != Type::Ordering {
                                self.emit(ErrorCode::TypeMismatch, "load() expects 1 ordering argument".into(), span);
                            }
                            return *inner;
                        }
                        "store" => {
                            if args.len() != 2 || !self.check_types_match(&inner, &args[0]) || arg_tys[1] != Type::Ordering {
                                self.emit(ErrorCode::TypeMismatch, format!("store() expects {:?} and 1 ordering argument", *inner), span);
                            }
                            return Type::Void;
                        }
                        "fetch_add" | "fetch_sub" | "fetch_and" | "fetch_or" | "fetch_xor" | "swap" => {
                            if args.len() != 2 || !self.check_types_match(&inner, &args[0]) || arg_tys[1] != Type::Ordering {
                                self.emit(ErrorCode::TypeMismatch, format!("{} expects {:?} and 1 ordering argument", method, *inner), span);
                            }
                            return *inner;
                        }
                        "compare_exchange" => {
                            if args.len() != 3 || !self.check_types_match(&inner, &args[0]) || !self.check_types_match(&inner, &args[1]) || arg_tys[2] != Type::Ordering {
                                // Simplified for Oxide: we take one ordering for success, and assume 1:1 C11 mapping
                                self.emit(ErrorCode::TypeMismatch, "compare_exchange expects 2 values and 1 ordering".into(), span);
                            }
                            return Type::Bool;
                        }
                        _ => {
                            self.emit(ErrorCode::UndefinedVariable, format!("Atomic type has no method '{}'", method), span);
                        }
                    }
                }
                Type::Inferred
            }
            Expr::FieldAccess(obj, field, span) => {
                let obj_ty = self.analyze_expr(obj);
                let resolved_ty = match obj_ty {
                    Type::Ref(inner) | Type::MutRef(inner) | Type::Ptr(inner) | Type::MutPtr(inner) => *inner,
                    _ => obj_ty,
                };
                
                if let Type::Struct(name) = resolved_ty {
                    if let Some(s) = self.structs.iter().find(|s| s.name == name).cloned() {
                        if let Some((_, f_ty, _)) = s.fields.iter().find(|(n, _, _)| n == field) {
                            return f_ty.clone();
                        }
                    }
                    self.emit(ErrorCode::UndefinedVariable, format!("Struct '{}' has no field '{}'", name, field), span);
                } else if let Type::Closure(_, _) = resolved_ty {
                    if field == "ptr_data" || field == "ptr_fn" {
                        return Type::USize;
                    }
                    self.emit(ErrorCode::UndefinedVariable, format!("Closure has no field '{}'", field), span);
                }
                Type::Inferred
            }
            Expr::Index(obj, index, span) => {
                let obj_ty = self.analyze_expr(obj);
                let idx_ty = self.analyze_expr(index);
                
                if idx_ty != Type::USize && idx_ty != Type::I32 && idx_ty != Type::I64 && idx_ty != Type::Inferred {
                    self.emit(ErrorCode::TypeMismatch, "Index must be an integer".into(), span);
                }
                
                match obj_ty {
                    Type::Array(inner, _) => *inner,
                    Type::Inferred => Type::Inferred,
                    _ => {
                        self.emit(ErrorCode::TypeMismatch, "Cannot index non-array type".into(), span);
                        Type::Inferred
                    }
                }
            }
            Expr::StructLit(name, fields, span) => {
                for (_, value) in fields {
                    self.analyze_expr(value);
                }
                if self.structs.iter().any(|s| s.name == *name) {
                    Type::Struct(name.clone())
                } else {
                    self.emit(ErrorCode::UndefinedType, format!("Undefined struct '{}'", name), span);
                    Type::Inferred
                }
            }
            Expr::Block(block) => {
                self.analyze_block(block);
                Type::Void
            }
            Expr::Path(_, _) => Type::Inferred,
            Expr::Cast(inner_expr, target_ty, _) => {
                let _inner_ty = self.analyze_expr(inner_expr);
                // Phase 1: Explicit 'as' casts safely bypass strict type bounds (e.g., &mut to *mut ptrs)
                self.resolve_type_expr(target_ty)
            },
            Expr::Abort(_, _span) => {
                Type::Inferred // abort safely halts execution, acts as Never type
            }
            Expr::Ordering(_, _) => Type::Ordering,
            Expr::Closure(params, body, span) => {
                self.enter_scope();
                let mut param_tys = Vec::new();
                for param in params {
                    // For closures, params might be inferred.
                    // In Phase 1, we'll assume they are inferred or we might have a way to specify them in future.
                    // For now, treat as Inferred.
                    let ty = Type::Inferred; 
                    self.define_var(param, ty.clone(), false, span);
                    param_tys.push(ty);
                }
                
                // Save outer return type context
                let prev_expected = self.expected_return_type.clone();
                // For closures in Phase 1, we relax return checking to allow inference
                self.expected_return_type = Some(Type::Inferred);

                // Analyze closure body
                self.analyze_block(body);
                
                // ... (Capture analysis)
                let captures = self.find_source_variables_block(body);
                for cap_name in captures {
                    if let Some(v) = self.lookup_var(&cap_name) {
                        if v.scope_depth < self.scope_depth {
                            // Variable is captured.
                        }
                    }
                }

                // TODO: Infer return type from body if not explicit.
                let ret_ty = Type::Void; 
                
                self.expected_return_type = prev_expected;
                self.exit_scope();
                Type::Closure(param_tys, Box::new(ret_ty))
            }
        }
    }

    fn check_types_match(&mut self, expected: &Type, actual_expr: &Expr) -> bool {
        let actual_ty = self.analyze_expr(actual_expr);
        if *expected == actual_ty || *expected == Type::Inferred || actual_ty == Type::Inferred {
            return true;
        }
        
        // Handle atomic wrapper matching
        if let Type::Atomic(inner) = expected {
            if self.check_types_match(inner, actual_expr) {
                return true;
            }
        }

        // Integer literal flexibility
        if matches!(actual_expr, Expr::IntLit(_, _)) {
            if expected.is_integer() {
                return true;
            }
        }

        // Closure matching
        if let Type::Closure(e_params, e_ret) = expected {
            if let Type::Closure(a_params, a_ret) = actual_ty {
                if e_params.len() == a_params.len() {
                    let params_match = e_params.iter().zip(a_params.iter())
                        .all(|(e, a)| e == a || *e == Type::Inferred || *a == Type::Inferred);
                    let ret_match = **e_ret == *a_ret || **e_ret == Type::Inferred || *a_ret == Type::Inferred;
                    return params_match && ret_match;
                }
            }
        }
        
        false
    }

    // ── Ownership & Borrow Checking ──

    /// MEMORY_MODEL Rule: Either multiple &T OR one &mut T
    fn check_mutable_borrow(&mut self, name: &str, span: &Span) {
        if let Some(var) = self.lookup_var_mut(name) {
            match &var.ownership {
                OwnershipState::Owned => {
                    var.ownership = OwnershipState::MutBorrowed;
                }
                OwnershipState::ImmutBorrowed(_) => {
                    self.emit(ErrorCode::MutAliasViolation,
                        format!("Cannot take &mut of '{}' while immutably borrowed", name), span);
                }
                OwnershipState::MutBorrowed => {
                    self.emit(ErrorCode::MutAliasViolation,
                        format!("Cannot take second &mut of '{}': already mutably borrowed", name), span);
                }
                OwnershipState::Moved => {
                    self.emit(ErrorCode::UseAfterMove,
                        format!("Cannot borrow moved value '{}'", name), span);
                }
            }
        }
    }

    fn check_immutable_borrow(&mut self, name: &str, span: &Span) {
        if let Some(var) = self.lookup_var_mut(name) {
            match &var.ownership {
                OwnershipState::Owned => {
                    var.ownership = OwnershipState::ImmutBorrowed(1);
                }
                OwnershipState::ImmutBorrowed(n) => {
                    let new_n = n + 1;
                    var.ownership = OwnershipState::ImmutBorrowed(new_n);
                }
                OwnershipState::MutBorrowed => {
                    self.emit(ErrorCode::MutAliasViolation,
                        format!("Cannot take & of '{}' while mutably borrowed", name), span);
                }
                OwnershipState::Moved => {
                    self.emit(ErrorCode::UseAfterMove,
                        format!("Cannot borrow moved value '{}'", name), span);
                }
            }
        }
    }

    // ── Region Escape Analysis ──

    /// Extracts all variable identifiers fundamentally involved in producing an expression's value
    fn find_source_variables(&self, expr: &Expr) -> Vec<String> {
        let mut vars = Vec::new();
        match expr {
            Expr::Ident(name, _) => vars.push(name.clone()),
            Expr::BinaryOp(l, _, r, _) => {
                vars.extend(self.find_source_variables(l));
                vars.extend(self.find_source_variables(r));
            }
            Expr::UnaryOp(_, operand, _) => vars.extend(self.find_source_variables(operand)),
            Expr::Call(callee, args, _) => {
                vars.extend(self.find_source_variables(callee));
                for arg in args { vars.extend(self.find_source_variables(arg)); }
            }
            Expr::MethodCall(recv, _, args, _) => {
                vars.extend(self.find_source_variables(recv));
                for arg in args { vars.extend(self.find_source_variables(arg)); }
            }
            Expr::FieldAccess(obj, _, _) => vars.extend(self.find_source_variables(obj)),
            Expr::Index(obj, idx, _) => {
                vars.extend(self.find_source_variables(obj));
                vars.extend(self.find_source_variables(idx));
            }
            Expr::StructLit(_, fields, _) => {
                for (_, val) in fields { vars.extend(self.find_source_variables(val)); }
            }
            Expr::Block(block) => {
                if let Some(Stmt::ExprStmt(e)) = block.stmts.last() {
                    vars.extend(self.find_source_variables(e));
                }
            }
            _ => {}
        }
        vars
    }

    fn find_source_variables_block(&self, block: &Block) -> Vec<String> {
        let mut vars = Vec::new();
        for stmt in &block.stmts {
            match stmt {
                Stmt::ExprStmt(expr) => vars.extend(self.find_source_variables(expr)),
                Stmt::Let(l) => {
                    if let Some(init) = &l.initializer {
                        vars.extend(self.find_source_variables(init));
                    }
                }
                Stmt::Return(expr, _) => {
                    if let Some(e) = expr {
                        vars.extend(self.find_source_variables(e));
                    }
                }
                Stmt::If(i) => {
                    vars.extend(self.find_source_variables(&i.condition));
                    vars.extend(self.find_source_variables_block(&i.then_block));
                    if let Some(eb) = &i.else_branch {
                        match eb.as_ref() {
                            ElseBranch::ElseIf(ni) => vars.extend(self.find_source_variables_block(&Block { stmts: vec![Stmt::If(ni.clone())], span: i.span.clone() })),
                            ElseBranch::Else(b) => vars.extend(self.find_source_variables_block(b)),
                        }
                    }
                }
                Stmt::Assignment(a) => {
                    vars.extend(self.find_source_variables(&a.value));
                }
                _ => {}
            }
        }
        vars
    }

    /// MEMORY_MODEL Rule: region-bound values cannot escape via return
    fn check_region_escape_expr(&mut self, expr: &Expr) {
        let sources = self.find_source_variables(expr);
        for name in sources {
            let region_name = self.lookup_var(&name)
                .and_then(|v| v.region.clone());
            if let Some(region) = region_name {
                if !self.in_unsafe {
                    self.emit(ErrorCode::RegionEscape,
                        format!("Value '{}' is bound to region '{}' and cannot escape via return",
                            name, region), &self.get_expr_span(expr));
                }
            }
        }
    }

    fn get_expr_span(&self, expr: &Expr) -> Span {
        match expr {
            Expr::Ident(_, s) | Expr::IntLit(_, s) | Expr::FloatLit(_, s) |
            Expr::StringLit(_, s) | Expr::CharLit(_, s) | Expr::BoolLit(_, s) |
            Expr::BinaryOp(_, _, _, s) | Expr::UnaryOp(_, _, s) |
            Expr::Call(_, _, s) | Expr::MethodCall(_, _, _, s) |
            Expr::FieldAccess(_, _, s) | Expr::Index(_, _, s) |
            Expr::Abort(_, s) | Expr::Ordering(_, s) |
            Expr::Closure(_, _, s) | Expr::Path(_, s) |
            Expr::StructLit(_, _, s) | Expr::Cast(_, _, s) => s.clone(),
            Expr::Block(b) => b.span.clone(),
        }
    }

    fn check_region_exit(&mut self, _region_name: &str, _span: &Span) {
        // At region exit, all variables in this region are invalidated.
        // The scope exit will handle variable cleanup.
        // This is where O(1) bulk deallocation semantics apply.
    }

    // ── Scope Management ──

    fn enter_scope(&mut self) {
        self.scope_depth += 1;
    }

    fn exit_scope(&mut self) {
        // Drop all variables at this scope depth (reverse order = deterministic destructors)
        self.variables.retain(|v| v.scope_depth < self.scope_depth);
        self.scope_depth -= 1;
    }

    fn define_var(&mut self, name: &str, ty: Type, is_mut: bool, span: &Span) {
        // Check for shadowing at same scope
        if self.variables.iter().any(|v| v.name == name && v.scope_depth == self.scope_depth) {
            self.emit(ErrorCode::DuplicateDefinition,
                format!("Variable '{}' already defined in this scope", name), span);
            return;
        }

        self.variables.push(VarInfo {
            name: name.to_string(),
            ty,
            is_mut,
            ownership: OwnershipState::Owned,
            region: self.current_region.clone(),
            scope_depth: self.scope_depth,
            def_span: span.clone(),
        });
    }

    fn lookup_var(&self, name: &str) -> Option<&VarInfo> {
        self.variables.iter().rev().find(|v| v.name == name)
    }

    fn lookup_var_mut(&mut self, name: &str) -> Option<&mut VarInfo> {
        self.variables.iter_mut().rev().find(|v| v.name == name)
    }

    // ── Type Resolution ──

    fn resolve_type_expr(&self, ty: &TypeExpr) -> Type {
        match ty {
            TypeExpr::Named(name, _) => {
                Type::from_name(name).unwrap_or_else(|| {
                    if self.structs.iter().any(|s| s.name == *name) {
                        Type::Struct(name.clone())
                    } else {
                        Type::Struct(name.clone()) // Allow forward references
                    }
                })
            }
            TypeExpr::Ref(inner, _) => Type::Ref(Box::new(self.resolve_type_expr(inner))),
            TypeExpr::MutRef(inner, _) => Type::MutRef(Box::new(self.resolve_type_expr(inner))),
            TypeExpr::Ptr(inner, _) => Type::Ptr(Box::new(self.resolve_type_expr(inner))),
            TypeExpr::MutPtr(inner, _) => Type::MutPtr(Box::new(self.resolve_type_expr(inner))),
            TypeExpr::Atomic(inner, _) => {
                let resolved = self.resolve_type_expr(inner);
                Type::Atomic(Box::new(resolved))
            }
            TypeExpr::Array(elem, _, _) => {
                Type::Array(Box::new(self.resolve_type_expr(elem)), 0) // size resolved later
            }
            TypeExpr::Closure(params, ret, _) => {
                let p_tys = params.iter().map(|p| self.resolve_type_expr(p)).collect();
                let r_ty = self.resolve_type_expr(ret);
                Type::Closure(p_tys, Box::new(r_ty))
            }
            TypeExpr::Unit(_) => Type::Void,
        }
    }

    fn is_deeply_immutable(&self, ty: &Type) -> bool {
        match ty {
            // Primitives are always immutable by value
            Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::USize |
            Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::ISize |
            Type::F32 | Type::F64 | Type::Bool | Type::Char | Type::Void => true,
            Type::Atomic(_) => true,
            Type::Ref(inner) => self.is_deeply_immutable(inner),
            Type::Array(inner, _) => self.is_deeply_immutable(inner),
            Type::Struct(name) => {
                // A struct is deeply immutable if all fields are deeply immutable
                if let Some(info) = self.structs.iter().find(|s| s.name == *name) {
                    info.fields.iter().all(|(_, ty, _)| self.is_deeply_immutable(ty))
                } else {
                    false
                }
            }
            // Mutable references/pointers are NOT immutable
            Type::MutRef(_) | Type::MutPtr(_) | Type::Ptr(_) => false,
            _ => false,
        }
    }

    // ── Error Emission ──

    fn emit(&mut self, code: ErrorCode, message: String, span: &Span) {
        self.errors.push(SemanticError {
            code,
            message,
            span: span.clone(),
        });
    }
}

// ============================================================
// Unit Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn analyze_source(source: &str) -> Analyzer {
        let tokens = Lexer::new(source).tokenize().expect("Lex failed");
        let program = Parser::new(tokens).parse_program().expect("Parse failed");
        let mut analyzer = Analyzer::new();
        analyzer.analyze(&program);
        analyzer
    }

    fn error_codes(source: &str) -> Vec<ErrorCode> {
        analyze_source(source).errors.iter().map(|e| e.code.clone()).collect()
    }

    // ── Ownership Tests ──

    #[test]
    fn test_valid_immutable_borrows() {
        // MEMORY_MODEL Test 1: Multiple &T borrows are valid
        let codes = error_codes("fn main() { let x = 10; let y = &x; let z = &x; }");
        assert!(codes.is_empty(), "Multiple immutable borrows should be valid");
    }

    #[test]
    fn test_mutable_alias_violation() {
        // MEMORY_MODEL Test 2: Two &mut T is forbidden
        let codes = error_codes("fn main() { let mut x = 42; let y = &mut x; let z = &mut x; }");
        assert!(codes.contains(&ErrorCode::MutAliasViolation),
            "Double mutable borrow should trigger E_MUT_ALIAS_VIOLATION");
    }

    #[test]
    fn test_shared_struct_atomic_valid() {
        // MEMORY_MODEL Test 8: shared struct with atomic field is valid
        let codes = error_codes("shared struct Counter { pub count: atomic<u64> } fn main() {}");
        assert!(codes.is_empty(), "Shared struct with atomic field should be valid");
    }

    #[test]
    fn test_shared_struct_non_atomic_rejected() {
        // MEMORY_MODEL Test 7: shared struct with non-atomic mutable field is invalid
        let codes = error_codes("shared struct Bad { pub value: &mut u32 } fn main() {}");
        assert!(codes.contains(&ErrorCode::SharedFieldNotAtomic),
            "Shared struct with &mut field should be rejected");
    }

    #[test]
    fn test_duplicate_function() {
        let codes = error_codes("fn foo() {} fn foo() {}");
        assert!(codes.contains(&ErrorCode::DuplicateDefinition));
    }

    #[test]
    fn test_duplicate_struct() {
        let codes = error_codes("struct A {} struct A {} fn main() {}");
        assert!(codes.contains(&ErrorCode::DuplicateDefinition));
    }

    #[test]
    fn test_immutable_assignment_rejected() {
        let codes = error_codes("fn main() { let x = 10; x = 20; }");
        assert!(codes.contains(&ErrorCode::TypeMismatch),
            "Assignment to immutable variable should be rejected");
    }

    #[test]
    fn test_type_mismatch_assignment() {
        let codes = error_codes("fn main() { let mut x: u32 = 10; x = true; }");
        assert!(codes.contains(&ErrorCode::TypeMismatch));
    }

    #[test]
    fn test_type_mismatch_return() {
        let codes = error_codes("fn get_val() -> u32 { return true; }");
        assert!(codes.contains(&ErrorCode::TypeMismatch));
    }

    #[test]
    fn test_mutable_assignment_valid() {
        let codes = error_codes("fn main() { let mut x = 10; x = 20; }");
        assert!(codes.is_empty(), "Assignment to mutable variable should be valid");
    }

    #[test]
    fn test_region_variable_scoping() {
        // Variables defined in region blocks should be scoped
        let a = analyze_source("fn main() { region r { let x = 42; } }");
        assert!(a.errors.is_empty());
    }

    #[test]
    fn test_region_escape_via_return() {
        // Check Rule XIII: Rejects return of value bound to inner region
        let codes = error_codes("fn escape() -> u32 { region r { let tmp = 99; return tmp; } }");
        assert!(codes.contains(&ErrorCode::RegionEscape), 
            "Returning a region-bound variable should trigger RegionEscape");
    }

    #[test]
    fn test_region_escape_via_assignment() {
        // Check Rule XIII: Rejects assignment of region-bound value to outer scope
        let codes = error_codes("fn assign_escape() { let mut global_val = 0; region r { let tmp = 10; global_val = tmp; } }");
        assert!(codes.contains(&ErrorCode::RegionEscape),
            "Assigning a region-bound variable to an outer scope should trigger RegionEscape");
    }

    #[test]
    fn test_shared_struct_immutable_field_valid() {
        // Deeply immutable primitive fields are allowed in shared structs
        let codes = error_codes("shared struct Config { pub max_size: u64 } fn main() {}");
        assert!(codes.is_empty(), "Shared struct with immutable primitive should be valid");
    }
    #[test]
    fn test_closure_analysis() {
        let source = "fn main() { let x = 42; let clos = |a| { let y = a + x; return y; }; }";
        let a = analyze_source(source);
        assert!(a.errors.is_empty(), "Closure analysis should succeed: {:?}", a.errors);
    }
}
