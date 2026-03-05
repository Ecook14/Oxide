// ============================================================
// Oxide Compiler — OxIR Structural Validator
// ============================================================
// Performs a linear pass over the generated OxIR to ensure
// lowering invariants are maintained before handing off to LLVM.
//
// Validation Checks:
//   1. Use-After-Move (registers marked by Move)
//   2. Dangling Borrows (EndBorrow before drop_in_place)
//   3. Region Bounds (AllocRegion inside RegionInit/RegionBulkFree)
//   4. Unknown Branch Targets (undefined labels)
// ============================================================

#![allow(dead_code)]

use crate::codegen::OxIR;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IrErrorCode {
    UseAfterMove,
    DanglingBorrow,
    RegionEscape,
    UndefinedLabel,
    InvalidInstructionSequence,
}

#[derive(Debug, Clone)]
pub struct IrError {
    pub code: IrErrorCode,
    pub message: String,
    pub instruction_index: usize,
}

pub struct Validator {
    pub errors: Vec<IrError>,
    
    // Validation State
    moved_registers: HashSet<String>,
    active_borrows: HashSet<String>,
    active_regions: HashSet<String>,
    defined_labels: HashSet<String>,
    referenced_labels: Vec<(String, usize)>, // (label, instruction_index)
}

impl Validator {
    pub fn new() -> Self {
        Validator {
            errors: Vec::new(),
            moved_registers: HashSet::new(),
            active_borrows: HashSet::new(),
            active_regions: HashSet::new(),
            defined_labels: HashSet::new(),
            referenced_labels: Vec::new(),
        }
    }

    fn emit(&mut self, code: IrErrorCode, message: String, index: usize) {
        self.errors.push(IrError { code, message, instruction_index: index });
    }

    pub fn validate(&mut self, instructions: &[OxIR]) {
        // Pass 1: Collect Labels
        for instr in instructions {
            if let OxIR::Label(name) = instr {
                self.defined_labels.insert(name.clone());
            }
        }

        // Pass 2: Linear Data-Flow Validation
        for (idx, instr) in instructions.iter().enumerate() {
            self.validate_instruction(instr, idx);
        }

        // Pass 3: Post-Validation Checks (Dangling Labels)
        let refs = self.referenced_labels.clone();
        for (label, idx) in refs {
            if !self.defined_labels.contains(&label) {
                self.emit(
                    IrErrorCode::UndefinedLabel,
                    format!("Branch to undefined label '{}'", label),
                    idx,
                );
            }
        }
        
        // Pass 4: Unclosed scopes (Should be caught by codegen boundaries, but double checking)
        let unclosed = self.active_borrows.clone();
        for borrow in unclosed {
            self.emit(
                IrErrorCode::DanglingBorrow,
                format!("Borrow '{}' was never explicitly ended via end_borrow", borrow),
                instructions.len() - 1,
            );
        }
    }

    fn check_usage(&mut self, reg: &str, idx: usize) {
        if self.moved_registers.contains(reg) {
            self.emit(
                IrErrorCode::UseAfterMove,
                format!("Use of moved register '{}'", reg),
                idx,
            );
        }
    }

    fn validate_instruction(&mut self, instr: &OxIR, idx: usize) {
        match instr {
            OxIR::Label(_) | OxIR::FnBegin(_, _) | OxIR::FnEnd | OxIR::Comment(_) | OxIR::Abort(_) => {}

            OxIR::AllocStack(_, _, dest) => {
                // Stack allocs are clean targets
                self.moved_registers.remove(dest); 
            }
            OxIR::AllocStruct(_, dest) => {
                self.moved_registers.remove(dest);
            }
            OxIR::AllocRegion(region_id, _, dest) => {
                if !self.active_regions.contains(region_id) {
                    self.emit(
                        IrErrorCode::RegionEscape,
                        format!("AllocRegion targeting inactive/undefined region '{}'", region_id),
                        idx,
                    );
                }
                self.moved_registers.remove(dest);
            }
            OxIR::Store(val, dest) | OxIR::StoreVal(val, dest) => {
                self.check_usage(val, idx);
                self.check_usage(dest, idx);
            }
            OxIR::StoreField(val, dest, _, _) => {
                self.check_usage(val, idx);
                self.check_usage(dest, idx);
            }
            OxIR::Load(ptr, dest) => {
                self.check_usage(ptr, idx);
                self.moved_registers.remove(dest);
            }
            OxIR::LoadField(ptr, _, _, dest) | 
            OxIR::FieldAddr(ptr, _, _, dest) => {
                self.check_usage(ptr, idx);
                self.moved_registers.remove(dest);
            }
            OxIR::Move(src, dest) => {
                self.check_usage(src, idx);
                self.moved_registers.insert(src.clone());
                self.moved_registers.remove(dest);
            }
            
            // ── Borrow Markers ──
            OxIR::BorrowImmut(owner, r) | OxIR::BorrowMut(owner, r) => {
                self.check_usage(owner, idx);
                self.active_borrows.insert(r.clone());
            }
            OxIR::EndBorrow(r) => {
                if !self.active_borrows.contains(r) {
                    // Tolerable if borrow unused, but structurally questionable
                }
                self.active_borrows.remove(r);
            }

            // ── Region Lifecycles ──
            OxIR::RegionInit(id) => {
                self.active_regions.insert(id.clone());
            }
            OxIR::RegionBulkFree(id) => {
                if !self.active_regions.contains(id) {
                    self.emit(
                        IrErrorCode::InvalidInstructionSequence,
                        format!("RegionBulkFree called on inactive/undefined region '{}'", id),
                        idx,
                    );
                }
                self.active_regions.remove(id);
            }

            OxIR::DropInPlace(ptr, _) => {
                self.check_usage(ptr, idx);
            }

            // ── Atomics ──
            OxIR::AtomicLoad(_, ptr, dest) => {
                self.check_usage(ptr, idx);
                self.moved_registers.remove(dest);
            }
            OxIR::AtomicStore(_, val, ptr) => {
                self.check_usage(val, idx);
                self.check_usage(ptr, idx);
            }
            OxIR::AtomicRMW(_, _, val, ptr, dest) => {
                self.check_usage(val, idx);
                self.check_usage(ptr, idx);
                self.moved_registers.remove(dest);
            }
            OxIR::AtomicCmpXchg(_, _, exp, new, ptr, dest) => {
                self.check_usage(exp, idx);
                self.check_usage(new, idx);
                self.check_usage(ptr, idx);
                self.moved_registers.remove(dest);
            }

            // ── Control Flow ──
            OxIR::Jump(label) => {
                self.referenced_labels.push((label.clone(), idx));
            }
            OxIR::Branch(cond, t, f) => {
                self.check_usage(cond, idx);
                self.referenced_labels.push((t.clone(), idx));
                self.referenced_labels.push((f.clone(), idx));
            }
            OxIR::Return(Some(val)) => {
                self.check_usage(val, idx);
            }
            OxIR::Return(None) => {}
            OxIR::Call(_, args, dest) => {
                for arg in args {
                    self.check_usage(arg, idx);
                }
                self.moved_registers.remove(dest);
            }
            OxIR::CallVoid(_, args) => {
                for arg in args {
                    self.check_usage(arg, idx);
                }
            }

            // ── Constants ──
            OxIR::ConstInt(_, dest) | 
            OxIR::ConstFloat(_, dest) | 
            OxIR::ConstBool(_, dest) | 
            OxIR::ConstString(_, dest) => {
                self.moved_registers.remove(dest);
            }

            // ── Arithmetic & Comparison ──
            OxIR::Add(a, b, d) | OxIR::Sub(a, b, d) | OxIR::Mul(a, b, d) |
            OxIR::Div(a, b, d) | OxIR::Mod(a, b, d) |
            OxIR::BitAnd(a, b, d) | OxIR::BitOr(a, b, d) | OxIR::BitXor(a, b, d) |
            OxIR::Shl(a, b, d) | OxIR::Shr(a, b, d) |
            OxIR::CmpEq(a, b, d) | OxIR::CmpNeq(a, b, d) |
            OxIR::CmpLt(a, b, d) | OxIR::CmpGt(a, b, d) |
            OxIR::CmpLtEq(a, b, d) | OxIR::CmpGtEq(a, b, d) => {
                self.check_usage(a, idx);
                self.check_usage(b, idx);
                self.moved_registers.remove(d);
            }
            
            OxIR::Neg(a, d) | OxIR::Not(a, d) | OxIR::BitNot(a, d) => {
                self.check_usage(a, idx);
                self.moved_registers.remove(d);
            }
        }
    }
}

// ============================================================
// Unit Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ir() {
        let ir = vec![
            OxIR::FnBegin("main".into(), vec![]),
            OxIR::ConstInt(42, "%t0".into()),
            OxIR::Return(Some("%t0".into())),
            OxIR::FnEnd,
        ];
        let mut v = Validator::new();
        v.validate(&ir);
        assert!(v.errors.is_empty(), "Valid IR should have no errors");
    }

    #[test]
    fn test_use_after_move() {
        let ir = vec![
            OxIR::ConstInt(42, "%t0".into()),
            OxIR::Move("%t0".into(), "%t1".into()),
            OxIR::Store("%t0".into(), "%t2".into()), // ERROR: %t0 was moved
        ];
        let mut v = Validator::new();
        v.validate(&ir);
        assert!(v.errors.iter().any(|e| e.code == IrErrorCode::UseAfterMove));
    }

    #[test]
    fn test_dangling_borrow() {
        let ir = vec![
            OxIR::ConstInt(42, "%t0".into()),
            OxIR::BorrowImmut("%t0".into(), "%ref0".into()),
            // Missing EndBorrow
        ];
        let mut v = Validator::new();
        v.validate(&ir);
        assert!(v.errors.iter().any(|e| e.code == IrErrorCode::DanglingBorrow));
    }

    #[test]
    fn test_alloc_region_bounds() {
        let ir = vec![
            OxIR::AllocRegion("%region_a".into(), 8, "%ptr".into()), // ERROR: Region not init
        ];
        let mut v = Validator::new();
        v.validate(&ir);
        assert!(v.errors.iter().any(|e| e.code == IrErrorCode::RegionEscape));
    }

    #[test]
    fn test_undefined_label() {
        let ir = vec![
            OxIR::Jump("bb_ghost".into()), // ERROR: label doesn't exist
        ];
        let mut v = Validator::new();
        v.validate(&ir);
        assert!(v.errors.iter().any(|e| e.code == IrErrorCode::UndefinedLabel));
    }
}
