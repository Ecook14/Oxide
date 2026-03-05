# OxIR Primitive Instruction Set v0.1

OxIR (Oxide Intermediate Representation) is the critical semantic bridge between the Oxide AST and LLVM IR. It exists to preserve ownership, region semantics, and atomic ordering information that would otherwise be lost in a direct-to-LLVM lowering process.

This document formally defines the Phase 0 OxIR primitive instruction set.

---

## I. OxIR Design Philosophy

- **High-Level Enough**: Retains region constraints, borrow lifetimes, and atomic intent.
- **Low-Level Enough**: Maps cleanly to register-based SSA (Static Single Assignment) form.
- **Deterministic**: Execution order, side effects, and memory drops are explicit.
- **Lowering Integrity**: OxIR to LLVM IR lowering must be a 1:1 structural translation of memory operations; LLVM is not permitted to "guess" memory safety.

---

## II. Type System Representation in OxIR

OxIR types are stripped of syntactic sugar but retain memory layout and concurrency tags.

- `i8, i16, i32, i64, isize` — standard integers
- `u8, u16, u32, u64, usize` — unsigned integers
- `f32, f64` — floats
- `ptr<T>` — raw unresolved pointer (unsafe)
- `ref<T, L>` — immutable borrow with lifetime `L`
- `mut_ref<T, L>` — mutable borrow with lifetime `L`
- `region_ptr<T, R>` — pointer bound cleanly to region `R`
- `atomic<T>` — memory location requiring explicit ordering

---

## III. Core Instruction Set

### 1. Memory Allocation & Initialization

- `alloc_stack %size, %align -> %ptr`
  - Allocates memory on the thread-local stack frame.
- `alloc_region %region_id, %size, %align -> %region_ptr`
  - Allocates memory tightly bound to `%region_id`. O(1) bulk free.
- `store %val, %ptr`
  - Standard memory write.
- `load %ptr -> %val`
  - Standard memory read.

### 2. Ownership & Moving

OxIR explicitly tracks ownership transfers.
- `move %src_ptr -> %dest_ptr`
  - Semantically copies the value from `%src_ptr` to `%dest_ptr` and immediately marks `%src_ptr` as `INVALID_USE_AFTER_MOVE` in the local flow graph.

### 3. Borrowing & Aliasing (Lexical Markers)

Borrow instructions do not emit assembly; they exist purely to enforce the Memory Model prior to LLVM lowering.

- `borrow_immut %owner_ptr, %lifetime -> %ref`
  - Initiates an immutable borrow.
- `borrow_mut %owner_ptr, %lifetime -> %mut_ref`
  - Initiates a mutable borrow. Asserts exclusive access statically.
- `end_borrow %ref`
  - Explicitly terminates a borrow lifetime.

### 4. Concurrency & Atomics

All atomic operations map directly to LLVM atomic instructions.

- `atomic_load [order] %ptr -> %val`
  - Options for `[order]`: `relaxed, acquire, seq_cst`
- `atomic_store [order] %val, %ptr`
  - Options for `[order]`: `relaxed, release, seq_cst`
- `atomic_rmw [op] [order] %val, %ptr -> %old_val`
  - `[op]`: `add, sub, and, or, xor, xchg`
  - Options for `[order]`: `relaxed, acquire, release, acq_rel, seq_cst`
- `atomic_cmpxchg [success_order] [failure_order] %expected, %new, %ptr -> { %val, %bool }`
  - Strong compare-and-swap primitive.

### 5. Deterministic Destructors

- `drop_in_place %ptr, %type`
  - Explicitly executes the destructor for `%type` at the exact location of `%ptr`. 
  - Emitted automatically by the frontend at the end of lexical scopes for non-region allocations.
- `region_bulk_free %region_id`
  - Emitted at the exit block of a `region { ... }` scope. 
  - Conceptually resets the region arena allocator pointer. No destructors are called internally.

### 6. Control Flow

OxIR utilizes standard SSA basic blocks.

- `jmp %block_id`
- `br %cond, %true_block, %false_block`
- `ret %val`
- `call %func, [%args...] -> %ret`

---

## IV. The Unsafe Boundary in OxIR

Unsafe operations in Oxide translate to unrestricted pointer mathematics in OxIR. They bypass borrow checking but are still structurally represented.

- `ptr_cast %ptr, %new_type -> %casted_ptr`
- `ptr_offset %ptr, %isize -> %new_ptr`
  - Corresponds to LLVM's GetElementPtr (GEP) instruction.

---

## V. Example: OxIR Lowering 

**Oxide Source:**
```oxide
fn counter() {
    let count = shared Counter { c: 0 };
    count.c.fetch_add(1, relaxed);
}
```

**OxIR Representation:**
```text
fn @counter() -> void {
bb0:
    // Allocate stack space for the shared Counter
    %count_ptr = alloc_stack 8, 8 
    
    // Initialize 0
    %init_val = const_i64 0
    store %init_val, %count_ptr
    
    // The shared semantic is structural, the operation is atomic
    %add_val = const_i64 1
    %old_val = atomic_rmw add relaxed %add_val, %count_ptr
    
    // Implicit determinism: Stack allocation popped
    ret void
}
```

**Oxide Source:**
```oxide
fn region_demo() {
    region r {
        let x = r.alloc(42);
    }
}
```

**OxIR Representation:**
```text
fn @region_demo() -> void {
bb0:
    // Region initialization (Arena pointer setup)
    %r_id = init_region
    
    // Allocate bounded to region %r_id
    %x_ptr = alloc_region %r_id, 8, 8
    
    %init_val = const_i64 42
    store %init_val, %x_ptr
    
    // Region explicitly bulk-freed at lexical exit
    region_bulk_free %r_id
    
    ret void
}
```

---

## VI. Invariants Guaranteed Before LLVM

Before OxIR is passed to the LLVM IR emitter, the `OxIR Validator Pass` ensures:
1. No `%ref` or `%mut_ref` outlives its `%owner_ptr` scope `drop_in_place`.
2. No memory marked `INVALID_USE_AFTER_MOVE` is accessed.
3. No `%region_ptr` escapes the basic blocks cleanly bounded by `region_bulk_free`.
4. No atomic ordering violations (e.g., using `release` on a `load`).
