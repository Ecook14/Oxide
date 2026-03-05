// ============================================================
// Oxide Compiler — C Backend Transpiler
// ============================================================
// Lowers validated OxIR instructions directly into C code,
// relying on gcc/clang for final ABI and optimization passes.
//
// This enables rapid bootstrapping and perfectly matches the
// C-ABI mandates of the `backend-team.md` without heavy
// LLVM build dependencies in Phase 1.
// ============================================================

#![allow(dead_code)]

use crate::codegen::{OxIR, MemoryOrder, AtomicOp};
use crate::ast::{Program, Item, TypeExpr};

pub struct CGenerator {
    pub output: String,
    pub c_structs: String,
    indent: usize,
    internal_structs: std::collections::HashMap<String, Vec<(String, String)>>,
}

impl CGenerator {
    pub fn new() -> Self {
        CGenerator {
            output: String::new(),
            c_structs: String::new(),
            indent: 0,
            internal_structs: std::collections::HashMap::new(),
        }
    }

    fn push(&mut self, line: &str) {
        let prefix = "    ".repeat(self.indent);
        self.output.push_str(&prefix);
        self.output.push_str(line);
        self.output.push('\n');
    }

    pub fn generate_structs(&mut self, program: &Program) {
        for item in &program.items {
            if let Item::Struct(s) = item {
                self.c_structs.push_str(&format!("typedef struct {{\n"));
                for field in &s.fields {
                    let c_type = self.map_c_type(&field.ty);
                    self.c_structs.push_str(&format!("    {} {};\n", c_type, field.name));
                }
                self.c_structs.push_str(&format!("}} {};\n\n", s.name));
            }
        }
    }

    fn map_c_type(&self, ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Named(name, _) => {
                match name.as_str() {
                    "u64" | "i64" | "usize" => "uint64_t".to_string(),
                    "u32" | "i32" => "uint32_t".to_string(),
                    "u16" | "i16" => "uint16_t".to_string(),
                    "u8"  | "i8"  => "uint8_t".to_string(),
                    "bool" => "bool".to_string(),
                    "f32" => "float".to_string(),
                    "f64" => "double".to_string(),
                    _ => name.clone(), // Custom struct nested
                }
            }
            TypeExpr::Ptr(inner, _) | TypeExpr::MutPtr(inner, _) | TypeExpr::Ref(inner, _) | TypeExpr::MutRef(inner, _) => {
                format!("{}*", self.map_c_type(inner))
            }
            TypeExpr::Atomic(inner, _) => {
                format!("_Atomic {}", self.map_c_type(inner))
            }
            _ => "uint64_t".to_string(), // fallback for phase 1 MVP arrays
        }
    }

    pub fn generate(&mut self, instructions: &[OxIR], program: &Program) {
        self.push("#include <stdint.h>");
        self.push("#include <stdbool.h>");
        self.push("#include <stdlib.h>");
        self.push("#include <stdio.h>");
        self.push("#include <stdatomic.h>");
        self.push("#include <pthread.h>");
        self.push("");
        self.push("typedef struct { uint64_t ptr_data; uint64_t ptr_fn; } Closure;");
        self.push("");
        
        let structs = self.c_structs.clone(); // fix borrow checker
        self.push(&structs);
        self.push("");
        
        // Define our custom struct for tuple returns like cmpxchg (unused in v0.1 simplification)
        // self.push("typedef struct { uint64_t val; bool ok; } cas_result;");
        self.push("");

        // Pre-pass: Generate forward declarations for all functions
        for instr in instructions {
            if let OxIR::FnBegin(name, args) = instr {
                let ret_ty = if name == "main" { "int" } else { "uint64_t" };
                let a = args.iter().map(|arg| format!("uint64_t {}", sanitize_reg(arg))).collect::<Vec<_>>().join(", ");
                self.push(&format!("{} {}({});", ret_ty, name, a));
            }
        }
        self.push("");

        // Pre-pass: Discover all internal structs
        for instr in instructions {
            match instr {
                OxIR::StoreField(_, _, ty, field) | OxIR::LoadField(_, ty, field, _) | OxIR::FieldAddr(_, ty, field, _) => {
                    self.ensure_internal_struct(ty, field);
                }
                _ => {}
            }
        }

        // Pre-render internal structs to avoid borrow checker conflicts
        let mut struct_defs = Vec::new();
        for (name, fields) in &self.internal_structs {
            if name == "Closure" { continue; } // Already defined at the top
            let mut s = format!("typedef struct {{\n");
            let mut seen_fields = std::collections::HashSet::new();
            for (f_name, f_type) in fields {
                if seen_fields.insert(f_name) {
                    s.push_str(&format!("    {} {};\n", f_type, f_name));
                }
            }
            s.push_str(&format!("}} {};", name));
            struct_defs.push(s);
        }
        for def in struct_defs {
            self.push(&def);
            self.push("");
        }
        self.push("");

        for instr in instructions {
            self.gen_instr(instr, program);
        }
    }

    fn ensure_internal_struct(&mut self, ty: &str, field: &str) {
        if ty.starts_with("_clos_env_") {
            let fields = self.internal_structs.entry(ty.to_string()).or_insert_with(Vec::new);
            if !fields.iter().any(|(f, _)| f == field) {
                fields.push((field.to_string(), "uint64_t".to_string()));
            }
        }
    }

    fn gen_instr(&mut self, instr: &OxIR, _program: &Program) {
        match instr {
            OxIR::FnBegin(name, args) => {
                let ret_ty = if name == "main" { "int" } else { "uint64_t" };
                let a = args.iter().map(|arg| format!("uint64_t {}", sanitize_reg(arg))).collect::<Vec<_>>().join(", ");
                self.push(&format!("{} {}({}) {{", ret_ty, name, a));
                self.indent += 1;
            }
            OxIR::FnEnd => {
                self.indent -= 1;
                self.push("}\n");
            }
            OxIR::Label(l) => {
                self.indent -= 1;
                self.push(&format!("{}:", l));
                self.indent += 1;
            }
            OxIR::Comment(c) => self.push(&format!("// {}", c)),

            // Memory
            OxIR::AllocStack(sz, _, dest) => {
                let d = sanitize_reg(dest);
                self.push(&format!("uint8_t {}[{}];", d, sz));
            }
            OxIR::AllocStruct(ty, dest) => {
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t {} = (uint64_t)calloc(1, sizeof({}));", d, ty));
            }
            OxIR::AllocRegion(_region, sz, dest) => {
                // Simplistic malloc for Region until Arena is built
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t {} = (uint64_t)malloc({});", d, sz));
            }
            OxIR::Store(val, ptr) | OxIR::StoreVal(val, ptr) => {
                let p = sanitize_reg(ptr);
                let v = sanitize_reg(val);
                self.push(&format!("*(uint64_t*){} = (uint64_t){};", p, v));
            }
            OxIR::StoreField(val, ptr, ty, field) => {
                let p = sanitize_reg(ptr);
                let v = sanitize_reg(val);
                self.ensure_internal_struct(ty, field);
                self.push(&format!("(({}*){})->{} = {};", ty, p, field, v));
            }
            OxIR::Load(ptr, dest) => {
                let p = sanitize_reg(ptr);
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t {} = *(uint64_t*){};", d, p));
            }
            OxIR::LoadField(ptr, ty, field, dest) => {
                let p = sanitize_reg(ptr);
                let d = sanitize_reg(dest);
                self.ensure_internal_struct(ty, field);
                self.push(&format!("uint64_t {} = (uint64_t)(({}*){})->{};", d, ty, p, field));
            }
            OxIR::FieldAddr(ptr, ty, field, dest) => {
                let p = sanitize_reg(ptr);
                let d = sanitize_reg(dest);
                self.ensure_internal_struct(ty, field);
                self.push(&format!("uint64_t {} = (uint64_t)&(({}*){})->{};", d, ty, p, field));
            }
            
            // Ownership (Zero-cost at C level)
            OxIR::Move(src, dest) => {
                let s = sanitize_reg(src);
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t {} = (uint64_t){};", d, s));
            }
            OxIR::BorrowImmut(src, dest) | OxIR::BorrowMut(src, dest) => {
                let s = sanitize_reg(src);
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t {} = (uint64_t){};", d, s));
            }
            OxIR::EndBorrow(_) => {}
            OxIR::DropInPlace(_, _) => {} // Simplistic Phase 1: assume trivial drop

            // Regions
            OxIR::RegionInit(_) => {} // In C transpiler, handled purely by stdlib
            OxIR::RegionBulkFree(_) => {} // Simulated garbage collection for C transpiler

            // Atomics (Mapping strictly to C11 stdatomic.h)
            OxIR::AtomicLoad(order, ptr, dest) => {
                let mo = map_memory_order(order);
                let p = sanitize_reg(ptr);
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t {} = atomic_load_explicit((_Atomic uint64_t*){}, {});", d, p, mo));
            }
            OxIR::AtomicStore(order, val, ptr) => {
                let mo = map_memory_order(order);
                let v = sanitize_reg(val);
                let p = sanitize_reg(ptr);
                self.push(&format!("atomic_store_explicit((_Atomic uint64_t*){}, {}, {});", p, v, mo));
            }
            OxIR::AtomicRMW(op, order, val, ptr, dest) => {
                let mo = map_memory_order(order);
                let fv = map_atomic_op(op);
                let v = sanitize_reg(val);
                let p = sanitize_reg(ptr);
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t {} = {}((_Atomic uint64_t*){}, {}, {});", d, fv, p, v, mo));
            }
            OxIR::AtomicCmpXchg(so, fo, exp, new, ptr, dest) => {
                let sorder = map_memory_order(so);
                let forder = map_memory_order(fo);
                let p = sanitize_reg(ptr);
                let e = sanitize_reg(exp);
                let n = sanitize_reg(new);
                let d = sanitize_reg(dest);
                self.push(&format!("uint64_t _exp_{} = {};", d, e));
                self.push(&format!("uint64_t {} = (uint64_t)atomic_compare_exchange_strong_explicit((_Atomic uint64_t*){}, &_exp_{}, {}, {}, {});", 
                    d, p, d, n, sorder, forder));
            }

            // Control flow
            OxIR::Jump(l) => self.push(&format!("goto {};", l)),
            OxIR::Branch(cond, t, f) => {
                let c = sanitize_reg(cond);
                self.push(&format!("if ({}) goto {}; else goto {};", c, t, f));
            }
            OxIR::Return(val) => {
                if let Some(v) = val {
                    let vr = sanitize_reg(v);
                    self.push(&format!("return {};", vr));
                } else {
                    self.push("return 0;"); // C requires main to return int
                }
            }
            OxIR::Call(func, args, dest) => {
                let a = args.iter().map(|arg| sanitize_reg(arg)).collect::<Vec<_>>();
                let d = sanitize_reg(dest);
                if func == "printf" {
                    if a.is_empty() {
                        self.push(&format!("int {} = printf();", d));
                    } else {
                        let fmt = &a[0];
                        let rest = if a.len() > 1 { format!(", {}", a[1..].join(", ")) } else { String::new() };
                        self.push(&format!("int {} = printf((const char*){}{});", d, fmt, rest));
                    }
                } else if func == "pthread_create" {
                    let args_str = format!("(pthread_t*){}, (const pthread_attr_t*){}, (void*(*)(void*)){}, (void*){}", a[0], a[1], a[2], a[3]);
                    self.push(&format!("uint64_t {} = (uint64_t){}({});", d, func, args_str));
                } else if func == "pthread_join" {
                    let args_str = format!("(pthread_t){}, (void**){}", a[0], a[1]);
                    self.push(&format!("uint64_t {} = (uint64_t){}({});", d, func, args_str));
                } else {
                    let f_name = if func.starts_with('%') {
                        // Cast to a generic function pointer type for the call
                        format!("((uint64_t(*)())({}))", sanitize_reg(func))
                    } else {
                        func.clone()
                    };
                    self.push(&format!("uint64_t {} = (uint64_t){}({});", d, f_name, a.join(", ")));
                }
            }
            OxIR::CallVoid(func, args) => {
                let a = args.iter().map(|arg| sanitize_reg(arg)).collect::<Vec<_>>();
                if func == "printf" {
                    if a.is_empty() {
                        self.push("printf();");
                    } else {
                        let fmt = &a[0];
                        let rest = if a.len() > 1 { format!(", {}", a[1..].join(", ")) } else { String::new() };
                        self.push(&format!("printf((const char*){}{});", fmt, rest));
                    }
                } else {
                    let f_name = if func.starts_with('%') {
                        format!("((void(*)())({}))", sanitize_reg(func))
                    } else {
                        func.clone()
                    };
                    self.push(&format!("{}({});", f_name, a.join(", ")));
                }
            }
            OxIR::Abort(msg) => {
                if let Some(m) = msg {
                    self.push(&format!("printf(\"\\n[Oxide Panic] %s\\n\", \"{}\");", m));
                } else {
                    self.push("printf(\"\\n[Oxide Panic] Explicit abort() invoked.\\n\");");
                }
                self.push("abort();");
            }

            // Constants
            OxIR::ConstInt(val, dest) => {
                self.push(&format!("uint64_t {} = {};", sanitize_reg(dest), val));
            }
            OxIR::ConstFloat(val, dest) => {
                self.push(&format!("double {} = {};", sanitize_reg(dest), val));
            }
            OxIR::ConstBool(val, dest) => {
                self.push(&format!("bool {} = {};", sanitize_reg(dest), val));
            }
            OxIR::ConstString(val, dest) => {
                let escaped = val.escape_default().to_string();
                self.push(&format!("const char* {} = \"{}\";", sanitize_reg(dest), escaped));
            }

            // Binary Ops
            OxIR::Add(a, b, d) => binop("+", a, b, d, &mut self.output, self.indent),
            OxIR::Sub(a, b, d) => binop("-", a, b, d, &mut self.output, self.indent),
            OxIR::Mul(a, b, d) => binop("*", a, b, d, &mut self.output, self.indent),
            OxIR::Div(a, b, d) => binop("/", a, b, d, &mut self.output, self.indent),
            OxIR::Mod(a, b, d) => binop("%", a, b, d, &mut self.output, self.indent),
            
            OxIR::CmpEq(a, b, d) => binop("==", a, b, d, &mut self.output, self.indent),
            OxIR::CmpNeq(a, b, d) => binop("!=", a, b, d, &mut self.output, self.indent),
            OxIR::CmpLt(a, b, d) => binop("<", a, b, d, &mut self.output, self.indent),
            OxIR::CmpGt(a, b, d) => binop(">", a, b, d, &mut self.output, self.indent),
            OxIR::CmpLtEq(a, b, d) => binop("<=", a, b, d, &mut self.output, self.indent),
            OxIR::CmpGtEq(a, b, d) => binop(">=", a, b, d, &mut self.output, self.indent),

            OxIR::BitAnd(a, b, d) => binop("&", a, b, d, &mut self.output, self.indent),
            OxIR::BitOr(a, b, d) => binop("|", a, b, d, &mut self.output, self.indent),
            OxIR::BitXor(a, b, d) => binop("^", a, b, d, &mut self.output, self.indent),
            OxIR::Shl(a, b, d) => binop("<<", a, b, d, &mut self.output, self.indent),
            OxIR::Shr(a, b, d) => binop(">>", a, b, d, &mut self.output, self.indent),

            // Unary Ops
            OxIR::Neg(a, d) => unop("-", a, d, &mut self.output, self.indent),
            OxIR::Not(a, d) => unop("!", a, d, &mut self.output, self.indent),
            OxIR::BitNot(a, d) => unop("~", a, d, &mut self.output, self.indent),
        }
    }
}

// Strip `%` from IR registers and handle internal names
fn sanitize_reg(s: &str) -> String {
    if s.starts_with('%') {
        s.replace("%", "r_")
    } else if s.starts_with('_') {
        s.to_string() // Internal names like _clos_tramp_0
    } else {
        format!("r_{}", s)
    }
}

fn map_memory_order(mo: &MemoryOrder) -> &'static str {
    match mo {
        MemoryOrder::Relaxed => "memory_order_relaxed",
        MemoryOrder::Acquire => "memory_order_acquire",
        MemoryOrder::Release => "memory_order_release",
        MemoryOrder::AcqRel => "memory_order_acq_rel",
        MemoryOrder::SeqCst => "memory_order_seq_cst",
    }
}

fn map_atomic_op(op: &AtomicOp) -> &'static str {
    match op {
        AtomicOp::Add => "atomic_fetch_add_explicit",
        AtomicOp::Sub => "atomic_fetch_sub_explicit",
        AtomicOp::And => "atomic_fetch_and_explicit",
        AtomicOp::Or => "atomic_fetch_or_explicit",
        AtomicOp::Xor => "atomic_fetch_xor_explicit",
        AtomicOp::Xchg => "atomic_exchange_explicit",
    }
}

fn binop(op: &str, a: &str, b: &str, d: &str, out: &mut String, indent: usize) {
    let prefix = "    ".repeat(indent);
    let l = sanitize_reg(a);
    let r = sanitize_reg(b);
    let res = sanitize_reg(d);
    
    // In Oxide v0.1, variables are stored in uint64_t indiscriminately.
    // To preserve signed semantics for comparisons and math, we cast to int64_t.
    if op == "<" || op == ">" || op == "<=" || op == ">=" || op == "/" || op == "%" {
        out.push_str(&format!("{}uint64_t {} = (int64_t){} {} (int64_t){};\n", prefix, res, l, op, r));
    } else {
        out.push_str(&format!("{}uint64_t {} = {} {} {};\n", prefix, res, l, op, r));
    }
}

fn unop(op: &str, a: &str, d: &str, out: &mut String, indent: usize) {
    let prefix = "    ".repeat(indent);
    let val = sanitize_reg(a);
    let res = sanitize_reg(d);
    out.push_str(&format!("{}uint64_t {} = {}{};\n", prefix, res, op, val));
}
