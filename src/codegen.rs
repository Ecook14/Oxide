// ============================================================
// Oxide Compiler — OxIR Code Generator
// ============================================================
// Translates AST → OxIR instructions.
// Maps directly to OXIR_SPEC_v0.1.md.
//
// OxIR preserves:
//   - Ownership transfers (move)
//   - Region boundaries (alloc_region, region_bulk_free)
//   - Borrow scopes (borrow_immut, borrow_mut, end_borrow)
//   - Atomic orderings (atomic_load, atomic_store, atomic_rmw)
//   - Deterministic destructors (drop_in_place)
// ============================================================

#![allow(dead_code)]

use crate::ast::*;

// ── OxIR Instructions ──

#[derive(Debug, Clone)]
pub enum OxIR {
    // ── Function structure ──
    FnBegin(String, Vec<String>),        // fn @name(args)
    FnEnd,
    Label(String),                       // bb0:

    // ── Memory ──
    AllocStack(usize, usize, String),    // alloc_stack size, align -> %name
    AllocStruct(String, String),         // alloc_struct TypeName -> %name
    AllocRegion(String, usize, String),  // alloc_region %region, size -> %name
    Store(String, String),               // store %val, %ptr (ptr to ptr)
    StoreVal(String, String),            // store_val %val, %ptr (val to ptr)
    StoreField(String, String, String, String),  // store_field %val, %struct_ptr, TypeName, field_name
    Load(String, String),                // load %ptr -> %dest
    LoadField(String, String, String, String),   // load_field %struct_ptr, TypeName, field_name -> %dest
    FieldAddr(String, String, String, String),   // field_addr %struct_ptr, TypeName, field_name -> %dest

    // ── Ownership ──
    Move(String, String),                // move %src -> %dest (invalidates src)

    // ── Borrowing (zero-cost markers) ──
    BorrowImmut(String, String),         // borrow_immut %owner -> %ref
    BorrowMut(String, String),           // borrow_mut %owner -> %mut_ref
    EndBorrow(String),                   // end_borrow %ref

    // ── Atomics ──
    AtomicLoad(MemoryOrder, String, String),  // atomic_load [order] %ptr -> %dest
    AtomicStore(MemoryOrder, String, String), // atomic_store [order] %val, %ptr
    AtomicRMW(AtomicOp, MemoryOrder, String, String, String), // atomic_rmw [op] [order] %val, %ptr -> %old
    AtomicCmpXchg(MemoryOrder, MemoryOrder, String, String, String, String), // cas

    // ── Destructors & Regions ──
    DropInPlace(String, String),         // drop_in_place %ptr, %type
    RegionInit(String),                  // init_region -> %region_id
    RegionBulkFree(String),              // region_bulk_free %region_id

    // ── Control flow ──
    Jump(String),                        // jmp %label
    Branch(String, String, String),      // br %cond, %true_label, %false_label
    Return(Option<String>),              // ret %val
    Call(String, Vec<String>, String),   // call %func, [args] -> %dest
    Abort(Option<String>),               // abort "msg"

    // ── Constants ──
    ConstInt(i64, String),               // const_i64 val -> %dest
    ConstFloat(f64, String),             // const_f64 val -> %dest
    ConstBool(bool, String),             // const_bool val -> %dest
    ConstString(String, String),         // const_str val -> %dest

    // ── Arithmetic (lowered from BinOp) ──
    Add(String, String, String),         // add %a, %b -> %dest
    Sub(String, String, String),
    Mul(String, String, String),
    Div(String, String, String),
    Mod(String, String, String),
    Neg(String, String),                 // neg %a -> %dest
    Not(String, String),
    BitAnd(String, String, String),
    BitOr(String, String, String),
    BitXor(String, String, String),
    Shl(String, String, String),
    Shr(String, String, String),
    BitNot(String, String),

    // ── Comparison ──
    CmpEq(String, String, String),
    CmpNeq(String, String, String),
    CmpLt(String, String, String),
    CmpGt(String, String, String),
    CmpLtEq(String, String, String),
    CmpGtEq(String, String, String),

    // ── Comment (for readability in OxIR dumps) ──
    Comment(String),
}

#[derive(Debug, Clone)]
pub enum MemoryOrder {
    Relaxed,
    Acquire,
    Release,
    AcqRel,
    SeqCst,
}

#[derive(Debug, Clone)]
pub enum AtomicOp {
    Add, Sub, And, Or, Xor, Xchg,
}

impl std::fmt::Display for MemoryOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryOrder::Relaxed => write!(f, "relaxed"),
            MemoryOrder::Acquire => write!(f, "acquire"),
            MemoryOrder::Release => write!(f, "release"),
            MemoryOrder::AcqRel => write!(f, "acq_rel"),
            MemoryOrder::SeqCst => write!(f, "seq_cst"),
        }
    }
}

impl std::fmt::Display for AtomicOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AtomicOp::Add  => write!(f, "add"),
            AtomicOp::Sub  => write!(f, "sub"),
            AtomicOp::And  => write!(f, "and"),
            AtomicOp::Or   => write!(f, "or"),
            AtomicOp::Xor  => write!(f, "xor"),
            AtomicOp::Xchg => write!(f, "xchg"),
        }
    }
}

// ── Code Generator ──

pub struct CodeGen {
    pub instructions: Vec<OxIR>,
    temp_counter: usize,
    label_counter: usize,
    loop_context: Vec<(String, String)>, // (continue_label, break_label)
    local_borrows: Vec<String>,
    functions: Vec<String>,
    closure_counter: usize,
    pending_closures: Vec<(String, String, Vec<String>, Vec<String>, Block)>, // (struct_name, tramp_name, captures, params, body)
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            instructions: Vec::new(),
            temp_counter: 0,
            label_counter: 0,
            loop_context: Vec::new(),
            local_borrows: Vec::new(),
            functions: Vec::new(),
            closure_counter: 0,
            pending_closures: Vec::new(),
        }
    }

    fn collect_identifiers_expr(&self, expr: &Expr, ids: &mut Vec<String>, bound: &Vec<String>) {
        match expr {
            Expr::Ident(name, _) => {
                if !bound.contains(name) && !ids.contains(name) {
                    ids.push(name.clone());
                }
            }
            Expr::BinaryOp(l, _, r, _) => {
                self.collect_identifiers_expr(l, ids, bound);
                self.collect_identifiers_expr(r, ids, bound);
            }
            Expr::UnaryOp(_, op, _) => self.collect_identifiers_expr(op, ids, bound),
            Expr::Call(callee, args, _) => {
                self.collect_identifiers_expr(callee, ids, bound);
                for a in args { self.collect_identifiers_expr(a, ids, bound); }
            }
            Expr::MethodCall(recv, _, args, _) => {
                self.collect_identifiers_expr(recv, ids, bound);
                for a in args { self.collect_identifiers_expr(a, ids, bound); }
            }
            Expr::FieldAccess(obj, _, _) => self.collect_identifiers_expr(obj, ids, bound),
            Expr::Index(obj, idx, _) => {
                self.collect_identifiers_expr(obj, ids, bound);
                self.collect_identifiers_expr(idx, ids, bound);
            }
            Expr::StructLit(_, fields, _) => {
                for (_, e) in fields { self.collect_identifiers_expr(e, ids, bound); }
            }
            Expr::Block(b) => self.collect_identifiers_block(b, ids, bound.clone()),
            Expr::Cast(e, _, _) => self.collect_identifiers_expr(e, ids, bound),
            _ => {}
        }
    }

    fn collect_identifiers_block(&self, block: &Block, ids: &mut Vec<String>, mut bound: Vec<String>) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::ExprStmt(e) => self.collect_identifiers_expr(e, ids, &bound),
                Stmt::Let(l) => {
                    // Only collect identifiers used in the INITIALIZER, not the variable name itself.
                    if let Some(init) = &l.initializer {
                        self.collect_identifiers_expr(init, ids, &bound);
                    }
                    bound.push(l.name.clone());
                }
                Stmt::Return(e, _) => {
                    if let Some(ex) = e { self.collect_identifiers_expr(ex, ids, &bound); }
                }
                Stmt::If(i) => {
                    self.collect_identifiers_expr(&i.condition, ids, &bound);
                    self.collect_identifiers_block(&i.then_block, ids, bound.clone());
                    if let Some(eb) = &i.else_branch {
                        match eb.as_ref() {
                            ElseBranch::ElseIf(ni) => {
                                let dummy_block = Block { stmts: vec![Stmt::If(ni.clone())], span: i.span.clone() };
                                self.collect_identifiers_block(&dummy_block, ids, bound.clone());
                            }
                            ElseBranch::Else(b) => self.collect_identifiers_block(b, ids, bound.clone()),
                        }
                    }
                }
                Stmt::Assignment(a) => {
                    if !matches!(a.target, Expr::Ident(_, _)) {
                        self.collect_identifiers_expr(&a.target, ids, &bound);
                    }
                    self.collect_identifiers_expr(&a.value, ids, &bound);
                }
                _ => {}
            }
        }
    }

    fn fresh_temp(&mut self) -> String {
        let name = format!("%t{}", self.temp_counter);
        self.temp_counter += 1;
        name
    }

    fn fresh_label(&mut self) -> String {
        let label = format!("bb{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    fn emit(&mut self, instr: OxIR) {
        self.instructions.push(instr);
    }

    // ── Public Entry ──

    pub fn generate(&mut self, program: &Program) {
        for item in &program.items {
            match item {
                Item::Function(f) => self.functions.push(f.name.clone()),
                Item::ExternFn(f) => self.functions.push(f.name.clone()),
                _ => {}
            }
        }
        for item in &program.items {
            if let Item::Function(f) = item {
                self.gen_function(f);
            }
        }

        // Process closures until none remain (closures can define closures)
        while !self.pending_closures.is_empty() {
            let pending = std::mem::take(&mut self.pending_closures);
            for (struct_name, tramp_name, captures, params, body) in pending {
                self.gen_trampoline(tramp_name, struct_name, captures, params, body);
            }
        }
    }

    fn gen_function(&mut self, f: &FunctionDecl) {
        self.local_borrows.clear();
        let args = f.params.iter().map(|p| format!("%arg_{}", p.name)).collect();
        self.emit(OxIR::FnBegin(f.name.clone(), args));
        let entry = self.fresh_label();
        self.emit(OxIR::Label(entry));

        // Allocate memory locations for arguments
        for param in &f.params {
            let var_name = format!("%{}", param.name);
            let arg_name = format!("%arg_{}", param.name);
            self.emit(OxIR::AllocStack(8, 8, var_name.clone()));
            self.emit(OxIR::StoreVal(arg_name, var_name));
        }

        for stmt in &f.body.stmts {
            self.gen_stmt(stmt);
        }

        // Implicit void return if no explicit return
        if !f.body.stmts.iter().any(|s| matches!(s, Stmt::Return(_, _))) {
            let borrows = self.local_borrows.clone();
            for b in borrows {
                self.emit(OxIR::EndBorrow(b));
            }
            self.emit(OxIR::Return(None));
        }

        self.emit(OxIR::FnEnd);
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                let var_name = format!("%{}", let_stmt.name);
                self.emit(OxIR::AllocStack(8, 8, var_name.clone()));

                if let Some(init) = &let_stmt.initializer {
                    let val = self.gen_expr(init);
                    self.emit(OxIR::Store(val, var_name));
                }
            }
            Stmt::Return(expr, _) => {
                let val = expr.as_ref().map(|e| self.gen_expr(e));
                let borrows = self.local_borrows.clone();
                for b in borrows {
                    self.emit(OxIR::EndBorrow(b));
                }
                self.emit(OxIR::Return(val));
            }
            Stmt::ExprStmt(expr) => {
                self.gen_expr(expr);
            }
            Stmt::If(if_stmt) => {
                self.gen_if(if_stmt);
            }
            Stmt::While(while_stmt) => {
                let cond_label = self.fresh_label();
                let body_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.emit(OxIR::Label(cond_label.clone()));
                let cond = self.gen_expr(&while_stmt.condition);
                self.emit(OxIR::Branch(cond, body_label.clone(), end_label.clone()));

                self.emit(OxIR::Label(body_label));
                
                self.loop_context.push((cond_label.clone(), end_label.clone()));
                for s in &while_stmt.body.stmts {
                    self.gen_stmt(s);
                }
                self.loop_context.pop();

                self.emit(OxIR::Jump(cond_label));

                self.emit(OxIR::Label(end_label));
            }
            Stmt::Loop(block, _) => {
                let loop_label = self.fresh_label();
                let end_label = self.fresh_label();

                self.emit(OxIR::Label(loop_label.clone()));
                
                self.loop_context.push((loop_label.clone(), end_label.clone()));
                for s in &block.stmts {
                    self.gen_stmt(s);
                }
                self.loop_context.pop();

                self.emit(OxIR::Jump(loop_label));
                self.emit(OxIR::Label(end_label));
            }
            Stmt::Break(_) => {
                if let Some((_, break_label)) = self.loop_context.last() {
                    self.emit(OxIR::Jump(break_label.clone()));
                } else {
                    self.emit(OxIR::Comment("break outside of loop".into()));
                }
            }
            Stmt::Continue(_) => {
                if let Some((continue_label, _)) = self.loop_context.last() {
                    self.emit(OxIR::Jump(continue_label.clone()));
                } else {
                    self.emit(OxIR::Comment("continue outside of loop".into()));
                }
            }
            Stmt::Region(region) => {
                let region_id = format!("%region_{}", region.name);
                self.emit(OxIR::Comment(format!("Region '{}' begins", region.name)));
                self.emit(OxIR::RegionInit(region_id.clone()));

                for s in &region.body.stmts {
                    self.gen_stmt(s);
                }

                // O(1) bulk deallocation
                self.emit(OxIR::RegionBulkFree(region_id));
                self.emit(OxIR::Comment(format!("Region '{}' freed (O(1))", region.name)));
            }
            Stmt::Unsafe(block, _) => {
                self.emit(OxIR::Comment("unsafe block begins".into()));
                for s in &block.stmts {
                    self.gen_stmt(s);
                }
                self.emit(OxIR::Comment("unsafe block ends".into()));
            }
            Stmt::Assignment(assign) => {
                let value = self.gen_expr(&assign.value);
                let target_reg = match &assign.target {
                    Expr::Ident(name, _) => format!("%{}", name),
                    _ => self.gen_expr(&assign.target)
                };

                let store_val = match assign.op {
                    AssignOp::Assign => value,
                    _ => {
                        let load_temp = self.fresh_temp();
                        self.emit(OxIR::Load(target_reg.clone(), load_temp.clone()));
                        let math_temp = self.fresh_temp();
                        
                        let instr = match assign.op {
                            AssignOp::AddAssign    => OxIR::Add(load_temp, value, math_temp.clone()),
                            AssignOp::SubAssign    => OxIR::Sub(load_temp, value, math_temp.clone()),
                            AssignOp::MulAssign    => OxIR::Mul(load_temp, value, math_temp.clone()),
                            AssignOp::DivAssign    => OxIR::Div(load_temp, value, math_temp.clone()),
                            AssignOp::BitAndAssign => OxIR::BitAnd(load_temp, value, math_temp.clone()),
                            AssignOp::BitOrAssign  => OxIR::BitOr(load_temp, value, math_temp.clone()),
                            AssignOp::BitXorAssign => OxIR::BitXor(load_temp, value, math_temp.clone()),
                            AssignOp::ShlAssign    => OxIR::Shl(load_temp, value, math_temp.clone()),
                            AssignOp::ShrAssign    => OxIR::Shr(load_temp, value, math_temp.clone()),
                            _ => unreachable!()
                        };
                        self.emit(instr);
                        math_temp
                    }
                };

                // For simple variable assignments, we know the register name
                let target_reg = match &assign.target {
                    Expr::Ident(name, _) => format!("%{}", name),
                    _ => self.gen_expr(&assign.target)
                };
                self.emit(OxIR::Store(store_val, target_reg));
            }
            _ => {}
        }
    }

    fn gen_if(&mut self, if_stmt: &IfStmt) {
        let then_label = self.fresh_label();
        let else_label = self.fresh_label();
        let end_label = self.fresh_label();

        let cond = self.gen_expr(&if_stmt.condition);
        self.emit(OxIR::Branch(cond, then_label.clone(), else_label.clone()));

        self.emit(OxIR::Label(then_label));
        for s in &if_stmt.then_block.stmts {
            self.gen_stmt(s);
        }
        self.emit(OxIR::Jump(end_label.clone()));

        self.emit(OxIR::Label(else_label));
        if let Some(else_branch) = &if_stmt.else_branch {
            match else_branch.as_ref() {
                ElseBranch::ElseIf(nested) => self.gen_if(nested),
                ElseBranch::Else(block) => {
                    for s in &block.stmts {
                        self.gen_stmt(s);
                    }
                }
            }
        }
        self.emit(OxIR::Jump(end_label.clone()));
        self.emit(OxIR::Label(end_label));
    }

    fn gen_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntLit(val, _) => {
                let dest = self.fresh_temp();
                self.emit(OxIR::ConstInt(*val, dest.clone()));
                dest
            }
            Expr::FloatLit(val, _) => {
                let dest = self.fresh_temp();
                self.emit(OxIR::ConstFloat(*val, dest.clone()));
                dest
            }
            Expr::BoolLit(val, _) => {
                let dest = self.fresh_temp();
                self.emit(OxIR::ConstBool(*val, dest.clone()));
                dest
            }
            Expr::StringLit(val, _) => {
                let dest = self.fresh_temp();
                self.emit(OxIR::ConstString(val.clone(), dest.clone()));
                dest
            }
            Expr::Ident(name, _) => {
                let dest = self.fresh_temp();
                if self.functions.contains(name) {
                    self.emit(OxIR::Move(name.clone(), dest.clone()));
                } else {
                    self.emit(OxIR::Load(format!("%{}", name), dest.clone()));
                }
                dest
            }
            Expr::BinaryOp(left, op, right, _) => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                let dest = self.fresh_temp();
                let instr = match op {
                    BinOp::Add   => OxIR::Add(l, r, dest.clone()),
                    BinOp::Sub   => OxIR::Sub(l, r, dest.clone()),
                    BinOp::Mul   => OxIR::Mul(l, r, dest.clone()),
                    BinOp::Div   => OxIR::Div(l, r, dest.clone()),
                    BinOp::Mod   => OxIR::Mod(l, r, dest.clone()),
                    BinOp::Eq    => OxIR::CmpEq(l, r, dest.clone()),
                    BinOp::Neq   => OxIR::CmpNeq(l, r, dest.clone()),
                    BinOp::Lt    => OxIR::CmpLt(l, r, dest.clone()),
                    BinOp::Gt    => OxIR::CmpGt(l, r, dest.clone()),
                    BinOp::LtEq  => OxIR::CmpLtEq(l, r, dest.clone()),
                    BinOp::GtEq  => OxIR::CmpGtEq(l, r, dest.clone()),
                    BinOp::And   => OxIR::BitAnd(l, r, dest.clone()),
                    BinOp::Or    => OxIR::BitOr(l, r, dest.clone()),
                    BinOp::BitAnd => OxIR::BitAnd(l, r, dest.clone()),
                    BinOp::BitOr  => OxIR::BitOr(l, r, dest.clone()),
                    BinOp::BitXor => OxIR::BitXor(l, r, dest.clone()),
                    BinOp::Shl   => OxIR::Shl(l, r, dest.clone()),
                    BinOp::Shr   => OxIR::Shr(l, r, dest.clone()),
                };
                self.emit(instr);
                dest
            }
            Expr::UnaryOp(op, operand, _) => {
                if matches!(op, UnaryOp::Ref | UnaryOp::MutRef) {
                    if let Expr::Ident(name, _) = operand.as_ref() {
                        let dest = self.fresh_temp();
                        let src = format!("%{}", name);
                        let instr = match op {
                            UnaryOp::Ref => OxIR::BorrowImmut(src, dest.clone()),
                            UnaryOp::MutRef => OxIR::BorrowMut(src, dest.clone()),
                            _ => unreachable!(),
                        };
                        self.emit(instr);
                        self.local_borrows.push(dest.clone());
                        return dest;
                    }
                }
                
                let val = self.gen_expr(operand);
                let dest = self.fresh_temp();
                let instr = match op {
                    UnaryOp::Neg    => OxIR::Neg(val, dest.clone()),
                    UnaryOp::Not    => OxIR::Not(val, dest.clone()),
                    UnaryOp::BitNot => OxIR::BitNot(val, dest.clone()),
                    UnaryOp::Ref    => { 
                        if let Expr::FieldAccess(obj, field, _) = operand.as_ref() {
                            let base = self.gen_expr(&obj);
                            let struct_name = "Unknown".to_string(); // Resolved in backend pre-pass typically
                            self.emit(OxIR::FieldAddr(base, struct_name, field.clone(), dest.clone()));
                        } else {
                            self.emit(OxIR::BorrowImmut(val, dest.clone())); 
                        }
                        self.local_borrows.push(dest.clone());
                        return dest; 
                    }
                    UnaryOp::MutRef => { 
                        self.emit(OxIR::BorrowMut(val, dest.clone())); 
                        self.local_borrows.push(dest.clone());
                        return dest; 
                    }
                    UnaryOp::Deref  => { self.emit(OxIR::Load(val, dest.clone())); return dest; }
                };
                self.emit(instr);
                dest
            }
            Expr::Call(callee, args, _) => {
                let mut is_direct = false;
                let mut callee_name = String::new();
                
                if let Expr::Ident(name, _) = callee.as_ref() {
                    if self.functions.contains(name) {
                        is_direct = true;
                        callee_name = name.clone();
                    }
                } else if let Expr::Path(path, _) = callee.as_ref() {
                    is_direct = true;
                    callee_name = path.last().unwrap().clone();
                }

                if is_direct {
                    let arg_regs: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                    let dest = self.fresh_temp();
                    self.emit(OxIR::Call(callee_name, arg_regs, dest.clone()));
                    dest
                } else {
                    // Closure call sequence: load from the Closure struct object
                    let arg_regs: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                    let clos_obj = self.gen_expr(callee);
                    
                    let ptr_data = self.fresh_temp();
                    let ptr_fn = self.fresh_temp();
                    
                    // Unpack from Closure struct
                    self.emit(OxIR::LoadField(clos_obj.clone(), "Closure".into(), "ptr_data".into(), ptr_data.clone()));
                    self.emit(OxIR::LoadField(clos_obj, "Closure".into(), "ptr_fn".into(), ptr_fn.clone()));
                    
                    // Construct new arg list: [ptr_data, ...original_args]
                    let mut closure_args = vec![ptr_data];
                    closure_args.extend(arg_regs);
                    
                    let dest = self.fresh_temp();
                    self.emit(OxIR::Call(ptr_fn, closure_args, dest.clone()));
                    dest
                }
            }
            Expr::MethodCall(receiver, method, args, _) => {
                let mut recv = self.gen_expr(receiver);
                
                // For atomic methods, we need the pointer to the variable, not its loaded value.
                // If the receiver was an identifier, gen_expr already emitted a LOAD.
                // We'll override 'recv' if it's an atomic method called on a direct variable.
                let is_atomic_method = matches!(method.as_str(), "load" | "store" | "fetch_add" | "fetch_sub" | "fetch_and" | "fetch_or" | "fetch_xor" | "swap" | "compare_exchange");
                if is_atomic_method {
                    if let Expr::Ident(name, _) = receiver.as_ref() {
                        recv = format!("%{}", name);
                    }
                }

                // Atomic method detection
                match method.as_str() {
                    "load" if args.len() == 1 => {
                        if let Expr::Ordering(ord, _) = &args[0] {
                            let dest = self.fresh_temp();
                            self.emit(OxIR::AtomicLoad(self.ast_to_ir_order(*ord), recv, dest.clone()));
                            return dest;
                        }
                    }
                    "store" if args.len() == 2 => {
                        if let Expr::Ordering(ord, _) = &args[1] {
                            let val = self.gen_expr(&args[0]);
                            self.emit(OxIR::AtomicStore(self.ast_to_ir_order(*ord), val, recv));
                            let dest = self.fresh_temp();
                            self.emit(OxIR::ConstInt(0, dest.clone()));
                            return dest;
                        }
                    }
                    "fetch_add" | "fetch_sub" | "fetch_and" | "fetch_or" | "fetch_xor" | "swap" if args.len() == 2 => {
                        if let Expr::Ordering(ord, _) = &args[1] {
                            let val = self.gen_expr(&args[0]);
                            let dest = self.fresh_temp();
                            let op = match method.as_str() {
                                "fetch_add" => AtomicOp::Add,
                                "fetch_sub" => AtomicOp::Sub,
                                "fetch_and" => AtomicOp::And,
                                "fetch_or"  => AtomicOp::Or,
                                "fetch_xor" => AtomicOp::Xor,
                                "swap"      => AtomicOp::Xchg,
                                _ => unreachable!(),
                            };
                            self.emit(OxIR::AtomicRMW(op, self.ast_to_ir_order(*ord), val, recv, dest.clone()));
                            return dest;
                        }
                    }
                    "compare_exchange" if args.len() == 3 => {
                         if let Expr::Ordering(ord, _) = &args[2] {
                            let expected = self.gen_expr(&args[0]);
                            let new = self.gen_expr(&args[1]);
                            let dest = self.fresh_temp();
                            // Oxide v0.1 simplification: same ordering for success/failure in OxIR
                            self.emit(OxIR::AtomicCmpXchg(self.ast_to_ir_order(*ord), self.ast_to_ir_order(*ord), expected, new, recv, dest.clone()));
                            return dest;
                        }
                    }
                    _ => {}
                }

                let arg_regs: Vec<String> = args.iter().map(|a| self.gen_expr(a)).collect();
                let dest = self.fresh_temp();
                let func = format!("{}.{}", recv, method);
                self.emit(OxIR::Call(func, arg_regs, dest.clone()));
                dest
            }
            Expr::StructLit(name, fields, _) => {
                let dest = self.fresh_temp();
                self.emit(OxIR::AllocStruct(name.clone(), dest.clone()));
                for (field_name, expr) in fields {
                    let v = self.gen_expr(expr);
                    self.emit(OxIR::StoreField(v, dest.clone(), name.clone(), field_name.clone()));
                }
                dest
            }
            Expr::FieldAccess(obj, field_name, _) => {
                let base = self.gen_expr(obj);
                let dest = self.fresh_temp();
                
                // Heuristic for closure and stdlib magic fields
                let struct_name = if field_name == "ptr_fn" || field_name == "ptr_data" {
                    "Closure".to_string()
                } else if field_name == "head" || field_name == "tail" || field_name == "buffer" || field_name == "capacity" || field_name == "sequences" {
                    "MpscQueue".to_string()
                } else if field_name == "current_ptr" || field_name == "max_size" || field_name == "region_start" {
                    "SlabAllocator".to_string()
                } else {
                    "Unknown".to_string()
                };
                
                self.emit(OxIR::LoadField(base, struct_name, field_name.clone(), dest.clone()));
                dest
            }
            Expr::Path(_, _) => {
                // Should only be hit if path used as standalone var, not supported currently
                let dest = self.fresh_temp();
                self.emit(OxIR::ConstInt(0, dest.clone()));
                dest
            }
            Expr::Cast(inner_expr, _ty, _) => {
                // OxIR is untyped in Phase 1 (all u64 registers)
                // Just evaluate the inner expression and return its register natively.
                self.gen_expr(inner_expr)
            }
            Expr::Abort(msg, _) => {
                self.emit(OxIR::Abort(msg.clone()));
                let empty_reg = self.fresh_temp();
                self.emit(OxIR::ConstInt(0, empty_reg.clone()));
                empty_reg
            }
            Expr::Ordering(_, _) => {
                let dest = self.fresh_temp();
                self.emit(OxIR::ConstInt(0, dest.clone()));
                dest
            }
            Expr::Closure(params, body, _) => {
                let id = self.closure_counter;
                self.closure_counter += 1;
                let struct_name = format!("_clos_env_{}", id);
                let tramp_name = format!("_clos_tramp_{}", id);
                
                let mut ids = Vec::new();
                self.collect_identifiers_block(body, &mut ids, Vec::new());
                
                let mut capture_set = std::collections::HashSet::new();
                for name in ids {
                    if !params.contains(&name) && !self.functions.contains(&name) {
                        capture_set.insert(name);
                    }
                }
                let captures: Vec<String> = capture_set.into_iter().collect();
                
                // Record for trampoline generation
                self.pending_closures.push((struct_name.clone(), tramp_name.clone(), captures.clone(), params.clone(), body.clone()));
                
                // Lowering:
                // 1. Alloc env struct
                let env_ptr = self.fresh_temp();
                self.emit(OxIR::AllocStruct(struct_name.clone(), env_ptr.clone()));
                
                // 2. Capture variables
                for cap in &captures {
                    let val = self.fresh_temp();
                    self.emit(OxIR::Load(format!("%{}", cap), val.clone()));
                    self.emit(OxIR::StoreField(val, env_ptr.clone(), struct_name.clone(), cap.clone()));
                }
                
                // 3. Return closure object (env_ptr + fn_ptr)
                let fn_ptr = self.fresh_temp();
                self.emit(OxIR::Move(tramp_name, fn_ptr.clone()));
                
                let clos_obj = self.fresh_temp();
                self.emit(OxIR::AllocStruct("Closure".into(), clos_obj.clone()));
                self.emit(OxIR::StoreField(env_ptr, clos_obj.clone(), "Closure".into(), "ptr_data".into()));
                self.emit(OxIR::StoreField(fn_ptr, clos_obj.clone(), "Closure".into(), "ptr_fn".into()));
                
                clos_obj
            }
            _ => {
                let dest = self.fresh_temp();
                self.emit(OxIR::Comment(format!("unhandled expr -> {}", dest)));
                dest
            }
        }
    }

    fn gen_trampoline(&mut self, name: String, struct_name: String, captures: Vec<String>, params: Vec<String>, body: Block) {
        let mut arg_regs = vec!["%__env".to_string()]; // Avoid collision with user params
        for p in &params {
            arg_regs.push(format!("%arg_{}", p));
        }

        self.emit(OxIR::FnBegin(name, arg_regs));
        let entry = self.fresh_label();
        self.emit(OxIR::Label(entry));

        // Unpack captures from the environment struct
        for cap in captures {
            let var_name = format!("%{}", cap);
            let val = self.fresh_temp();
            // If it's already a parameter, we don't need to capture/redeclare it
            if params.contains(&cap) { continue; }

            self.emit(OxIR::AllocStack(8, 8, var_name.clone()));
            self.emit(OxIR::LoadField("%__env".into(), struct_name.clone(), cap, val.clone()));
            self.emit(OxIR::StoreVal(val, var_name));
        }

        // Setup parameters as local variables
        for p in params {
            let var_name = format!("%{}", p);
            let arg_name = format!("%arg_{}", p);
            self.emit(OxIR::AllocStack(8, 8, var_name.clone()));
            self.emit(OxIR::StoreVal(arg_name, var_name));
        }

        for stmt in &body.stmts {
            self.gen_stmt(stmt);
        }

        // Implicit void return
        if !body.stmts.iter().any(|s| matches!(s, Stmt::Return(_, _))) {
            self.emit(OxIR::Return(None));
        }

        self.emit(OxIR::FnEnd);
    }

    fn ast_to_ir_order(&self, ord: crate::ast::MemoryOrdering) -> MemoryOrder {
        match ord {
            crate::ast::MemoryOrdering::Relaxed => MemoryOrder::Relaxed,
            crate::ast::MemoryOrdering::Acquire => MemoryOrder::Acquire,
            crate::ast::MemoryOrdering::Release => MemoryOrder::Release,
            crate::ast::MemoryOrdering::AcqRel  => MemoryOrder::AcqRel,
            crate::ast::MemoryOrdering::SeqCst  => MemoryOrder::SeqCst,
        }
    }

    // ── Pretty Printer ──

    pub fn dump(&self) -> String {
        let mut out = String::new();
        for instr in &self.instructions {
            match instr {
                OxIR::FnBegin(name, args) => out.push_str(&format!("fn @{}({}) {{\n", name, args.join(", "))),
                OxIR::FnEnd => out.push_str("}\n\n"),
                OxIR::Label(l) => out.push_str(&format!("{}:\n", l)),
                OxIR::Comment(c) => out.push_str(&format!("    // {}\n", c)),

                OxIR::AllocStack(sz, align, dest) => out.push_str(&format!("    {} = alloc_stack {}, {}\n", dest, sz, align)),
                OxIR::AllocStruct(ty, dest) => out.push_str(&format!("    {} = alloc_struct {}\n", dest, ty)),
                OxIR::AllocRegion(region, sz, dest) => out.push_str(&format!("    {} = alloc_region {}, {}\n", dest, region, sz)),
                OxIR::Store(val, ptr) => out.push_str(&format!("    store {}, {}\n", val, ptr)),
                OxIR::StoreVal(val, ptr) => out.push_str(&format!("    store_val {}, {}\n", val, ptr)),
                OxIR::StoreField(val, ptr, ty, field) => out.push_str(&format!("    store_field {}, {}.{} [{}]\n", val, ptr, field, ty)),
                OxIR::LoadField(ptr, ty, field, dest) => out.push_str(&format!("    {} = load_field {}.{} [{}]\n", dest, ptr, field, ty)),
                OxIR::FieldAddr(ptr, ty, field, dest) => out.push_str(&format!("    {} = field_addr {}.{} [{}]\n", dest, ptr, field, ty)),
                OxIR::Load(ptr, dest) => out.push_str(&format!("    {} = load {}\n", dest, ptr)),

                OxIR::Move(src, dest) => out.push_str(&format!("    {} = move {} [INVALIDATES {}]\n", dest, src, src)),

                OxIR::BorrowImmut(owner, bref) => out.push_str(&format!("    {} = borrow_immut {}\n", bref, owner)),
                OxIR::BorrowMut(owner, bref) => out.push_str(&format!("    {} = borrow_mut {}\n", bref, owner)),
                OxIR::EndBorrow(bref) => out.push_str(&format!("    end_borrow {}\n", bref)),

                OxIR::AtomicLoad(order, ptr, dest) => out.push_str(&format!("    {} = atomic_load [{}] {}\n", dest, order, ptr)),
                OxIR::AtomicStore(order, val, ptr) => out.push_str(&format!("    atomic_store [{}] {}, {}\n", order, val, ptr)),
                OxIR::AtomicRMW(op, order, val, ptr, dest) => out.push_str(&format!("    {} = atomic_rmw {} [{}] {}, {}\n", dest, op, order, val, ptr)),
                OxIR::AtomicCmpXchg(s, f, exp, new, ptr, dest) => out.push_str(&format!("    {} = atomic_cas [{}, {}] {}, {}, {}\n", dest, s, f, ptr, exp, new)),

                OxIR::DropInPlace(ptr, ty) => out.push_str(&format!("    drop_in_place {}, {}\n", ptr, ty)),
                OxIR::RegionInit(id) => out.push_str(&format!("    {} = init_region\n", id)),
                OxIR::RegionBulkFree(id) => out.push_str(&format!("    region_bulk_free {}\n", id)),

                OxIR::Jump(label) => out.push_str(&format!("    jmp {}\n", label)),
                OxIR::Branch(cond, t, f) => out.push_str(&format!("    br {}, {}, {}\n", cond, t, f)),
                OxIR::Return(val) => out.push_str(&format!("    ret {}\n", val.as_deref().unwrap_or("void"))),
                OxIR::Call(func, args, dest) => out.push_str(&format!("    {} = call {}({})\n", dest, func, args.join(", "))),
                OxIR::Abort(msg) => out.push_str(&format!("    abort {}\n", msg.as_deref().unwrap_or("[]"))),

                OxIR::ConstInt(v, dest) => out.push_str(&format!("    {} = const_i64 {}\n", dest, v)),
                OxIR::ConstFloat(v, dest) => out.push_str(&format!("    {} = const_f64 {}\n", dest, v)),
                OxIR::ConstBool(v, dest) => out.push_str(&format!("    {} = const_bool {}\n", dest, v)),
                OxIR::ConstString(v, dest) => out.push_str(&format!("    {} = const_str \"{}\"\n", dest, v)),

                OxIR::Add(a, b, d) => out.push_str(&format!("    {} = add {}, {}\n", d, a, b)),
                OxIR::Sub(a, b, d) => out.push_str(&format!("    {} = sub {}, {}\n", d, a, b)),
                OxIR::Mul(a, b, d) => out.push_str(&format!("    {} = mul {}, {}\n", d, a, b)),
                OxIR::Div(a, b, d) => out.push_str(&format!("    {} = div {}, {}\n", d, a, b)),
                OxIR::Mod(a, b, d) => out.push_str(&format!("    {} = mod {}, {}\n", d, a, b)),
                OxIR::Neg(a, d) => out.push_str(&format!("    {} = neg {}\n", d, a)),
                OxIR::Not(a, d) => out.push_str(&format!("    {} = not {}\n", d, a)),
                OxIR::BitAnd(a, b, d) => out.push_str(&format!("    {} = bit_and {}, {}\n", d, a, b)),
                OxIR::BitOr(a, b, d) => out.push_str(&format!("    {} = bit_or {}, {}\n", d, a, b)),
                OxIR::BitXor(a, b, d) => out.push_str(&format!("    {} = bit_xor {}, {}\n", d, a, b)),
                OxIR::Shl(a, b, d) => out.push_str(&format!("    {} = shl {}, {}\n", d, a, b)),
                OxIR::Shr(a, b, d) => out.push_str(&format!("    {} = shr {}, {}\n", d, a, b)),
                OxIR::BitNot(a, d) => out.push_str(&format!("    {} = bit_not {}\n", d, a)),

                OxIR::CmpEq(a, b, d) => out.push_str(&format!("    {} = cmp_eq {}, {}\n", d, a, b)),
                OxIR::CmpNeq(a, b, d) => out.push_str(&format!("    {} = cmp_neq {}, {}\n", d, a, b)),
                OxIR::CmpLt(a, b, d) => out.push_str(&format!("    {} = cmp_lt {}, {}\n", d, a, b)),
                OxIR::CmpGt(a, b, d) => out.push_str(&format!("    {} = cmp_gt {}, {}\n", d, a, b)),
                OxIR::CmpLtEq(a, b, d) => out.push_str(&format!("    {} = cmp_lteq {}, {}\n", d, a, b)),
                OxIR::CmpGtEq(a, b, d) => out.push_str(&format!("    {} = cmp_gteq {}, {}\n", d, a, b)),
            }
        }
        out
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

    fn gen(source: &str) -> String {
        let tokens = Lexer::new(source).tokenize().expect("Lex failed");
        let program = Parser::new(tokens).parse_program().expect("Parse failed");
        let mut codegen = CodeGen::new();
        codegen.generate(&program);
        codegen.dump()
    }

    #[test]
    fn test_simple_function() {
        let ir = gen("fn main() { let x = 42; }");
        assert!(ir.contains("fn @main()"));
        assert!(ir.contains("alloc_stack"));
        assert!(ir.contains("const_i64 42"));
        assert!(ir.contains("store"));
        assert!(ir.contains("ret void"));
    }

    #[test]
    fn test_arithmetic() {
        let ir = gen("fn add(a: u32, b: u32) -> u32 { return a + b; }");
        assert!(ir.contains("fn @add(%arg_a, %arg_b)"));
        assert!(ir.contains("add"));
        assert!(ir.contains("ret"));
    }

    #[test]
    fn test_region_ir() {
        let ir = gen("fn main() { region r { let x = 10; } }");
        assert!(ir.contains("init_region"));
        assert!(ir.contains("region_bulk_free"));
        assert!(ir.contains("Region 'r' begins"));
        assert!(ir.contains("Region 'r' freed (O(1))"));
    }

    #[test]
    fn test_unsafe_ir() {
        let ir = gen("fn main() { unsafe { let x = 1; } }");
        assert!(ir.contains("unsafe block begins"));
        assert!(ir.contains("unsafe block ends"));
    }

    #[test]
    fn test_conditionals_ir() {
        let ir = gen("fn main() { if true { let x = 1; } }");
        assert!(ir.contains("br"));
        assert!(ir.contains("jmp"));
    }
}
