# Oxide Memory Model v0.1: Ownership & Region Semantics

This document formally defines the final memory and concurrency semantics of Oxide. It serves as the source of truth for the type system, the compiler's safety checks, and OxIR's lowering boundaries.

---

## I. Core Ownership Model

Oxide uses **Single Ownership with Explicit Transfer**.
- Every value has exactly **one owner**.
- Ownership moves on assignment unless explicitly borrowed.
- No implicit sharing.
- No reference counting in the core language.
- No lifetime annotations in syntax.
- Unlike Rust:
  - No explicit lifetime parameters exposed to the user.
  - Ownership rules are structural and purely local.

---

## II. Region Semantics (Primary Allocation Model)

Regions are **lexically scoped memory arenas**.
- A region is a bounded memory context that:
  - Allocates memory linearly or via slab.
  - Frees all allocations at region exit.
  - Is thread-local unless explicitly marked `shared`.

**Region Rules:**
- A region cannot outlive its lexical scope.
- A value allocated in region `R` cannot escape `R`.
- The compiler enforces **escape analysis** locally.
- Cross-thread region access is forbidden unless explicitly marked `shared`.

**Region Exit Guarantee:**
- When a region ends:
  - All allocations are freed instantly.
  - **No destructors** are called for region-managed memory.
  - Provides deterministic **O(1) cleanup**.
- This guarantees zero fragmentation, zero per-object free cost, and constant-time bulk deallocations.

---

## III. Deterministic Destructors

For non-region-managed objects (e.g., standard stack or global scope):
- **Destructors run at scope exit.**
- Execution order is always the **reverse of construction**.
- Destructors **must not panic**.
- Destructors **must not allocate implicitly**.

**Destructor Invocation Rules:**
1. Stack-allocated objects → `drop` called at scope end.
2. Heap-allocated objects outside regions → `drop` called when the owner goes out of scope.
3. Region-owned objects bypass the destructor (unless explicitly opted-in via marker).

---

## IV. Borrowing Model (Simplified)

Oxide enforces an explicitly local borrowing model with two kinds:
- **Immutable borrow** (`&T`)
- **Mutable borrow** (`&mut T`)

**Rules:**
- Either multiple immutable borrows OR one mutable borrow (XOR).
- A borrow must not outlive its owner.
- Borrow tracking is purely **lexical**.
- No lifetime annotations are required, solving the borrow graph locally rather than globally.

---

## V. Shared Memory Semantics

Shared memory must be explicitly structurally declared:
```oxide
shared struct Cache { ... }
```

**Rules:**
- Shared types must contain ONLY:
  - `atomic<T>`
  - immutable fields
  - explicitly `unsafe` blocks
- **Non-atomic mutable fields are absolutely forbidden** in a shared context.
- Shared memory implicitly enforces boundary explicit synchronization.
- No underlying Send/Sync generic traits like Rust. Sharedness is strictly structural.

---

## VI. Thread-Local Guarantee

By default, the global assumption is:
- **All data is thread-local.**
- No value is cross-thread shareable unless declared `shared`.
- Regions are thread-local unless marked `shared`.
- Completely eliminates accidental data races in safe code.

---

## VII. Atomic Semantics

Atomics are first-class, requiring explicit memory ordering.
- `relaxed`, `acquire`, `release`, `acq_rel`, `seq_cst`.
- No implicit magic. The user specifies what the algorithm demands.
- Ordering semantics mirror the **C11 model**, enabling 1:1 ABI interop and stability without surprises.

---

## VIII. Unsafe Boundary

The `unsafe { ... }` block represents an explicit escape hatch context.

**Allows:**
- Raw pointer arithmetic.
- Manual memory manipulation and casting.
- Cross-region escapes.
- Manual synchronization primitives.

**Requires (from the Programmer):**
- Does NOT allow violating ABI rules or breaking the region/compiler invariants silently.
- The programmer assumes absolute responsibility to ensure no data races, use-after-free, or invalid dereferences occur.

---

## IX. Undefined Behavior (UB) Definition

UB occurs if and only if:
1. Dereferencing an invalid, unaligned, or null pointer.
2. A data race occurs on non-atomic memory.
3. Escaping a region-bound value beyond its lexical region manually.
4. Performing a double-free outside of the region model.
5. Violating atomic ordering hardware contracts.

Safe code must **never** produce UB.

---

## X. Escape Analysis Requirements (Compiler Constraints)

**Compiler MUST:**
- Prevent region escape statically.
- Prevent mutable aliasing statically.
- Validate borrow lifetimes lexically.
- Enforce structural `shared` restrictions.

**Compiler MUST NOT:**
- Require global lifetime graph solving equivalent to Rust.
- Introduce complex lifetime syntax.

---

## XI. Interaction with OxIR

Ownership metadata is a first-class citizen of OxIR.
- OxIR precisely encodes ownership transfer, region boundaries, borrow scopes, and atomic operations before lowering.
- Lowering to LLVM must completely respect semantic boundaries.

---
---

# Formal Semantic Definitions

To transition from conceptual semantics to compiler rules, we formally define the escape and validation constraints.

## XII. Borrow Validity Inference Algorithm

Let $\Gamma$ be the compiler environment assigning a lexical scope boundary $S(x)$ to every bound variable $x$.

**Algorithm (Compile-Time Validation):**
1. **Scope Capture**: For each local variable $x$, map $x \rightarrow S(x)$ based on index markers of scope entry and scope exit.
2. **Borrow Origination**: For every borrow $B_i = (\&x)$ or $B_i = (\&mut\ x)$ allocated at line $i$ and used up to line $j$, assign its lifetime scope $L_{B_i} = [i, j]$.
3. **Liveness Validation**: Enforce that $L_{B_i} \subseteq S(x)$. If FALSE, emit semantic conflict `E_BORROW_OUTLIVES_OWNER`.
4. **Lexical Aliasing Check**: For any variable $x$, identify all overlapping borrow lifetimes. If $L_{B_1} \cap L_{B_2} \neq \emptyset$:
   - If either is `&mut`, emit `E_MUT_ALIAS_VIOLATION`.
   - Otherwise, proceed.
5. **Move Invalidations**: If $x$ is moved at line $k$, implicitly terminate $S(x)$ at $k-1$. If any $L_{B} \cap [k, \infty) \neq \emptyset$, emit `E_USE_AFTER_MOVE`.

## XIII. Formal Region Escape Rules

Let $R_{local}$ be the implicitly or explicitly created lexical region belonging to function or block $F$.
Let $v$ be a dynamically allocated value bound to $R_{local}$.

**Escape Rule Constraint:**
A statement returning $v$ or assigning $v$ to a region $R_{outer}$ is only valid if $R_{local} \subset R_{outer}$ is mathematically provable at AST level. 

If $R_{local}$ is equal to or tighter than the lexical scope of the assignment target, the compiler emits a `Region Escape Violation`. All allocations within $R_{local}$ are rigidly tethered to $R_{local}$’s zero-cost deallocation phase.

---

## XIV. Reference Programs: 10 Edge-Case Validations

These small Oxide code fragments validate our Memory Model semantics.

### 1. Safe Multiple Immutable Borrows
```oxide
fn safe_immut_borrow() {
    let x = 10;
    let y = &x;
    let z = &x;
    // VALID: Reader locks allow multiple consumers simultaneously.
    print(y + z); 
}
```

### 2. Mutable Aliasing Violation (Blocked)
```oxide
fn mut_alias() {
    let mut x = 42;
    let y = &mut x;
    let z = &mut x; // ERROR: E_MUT_ALIAS_VIOLATION
    *y += 1;
}
```

### 3. Move Ownership Invalidating Outstanding Borrows (Blocked)
```oxide
fn move_invalidates() {
    let data = "buffer";
    let b = &data;
    let data_new = data; // Ownership formally transferred.
    print(b); // ERROR: E_USE_AFTER_MOVE
}
```

### 4. Returning Local Reference (Escape Blocked)
```oxide
fn escape_local() -> &int {
    let x = 99;
    return &x; // ERROR: Region Escape Violation. 'x' belongs to local region.
}
```

### 5. Safe Local Region Allocation (Valid)
```oxide
fn region_alloc() {
    region r { // explicit region scope
        let tmp = r.alloc(10);
        print(tmp);
    } // tmp deallocated implicitly with 0 overhead here.
}
```

### 6. Value Escaping Region Block (Blocked)
```oxide
fn region_violation() {
    let global_ptr;
    region r {
        let tmp = r.alloc(10);
        global_ptr = tmp; // ERROR: Value from region 'r' escaping to outer scope
    }
}
```

### 7. Explicitly Structured Shared Type (Blocked)
```oxide
shared struct BadState {
    value: int // ERROR: Shared structs cannot contain non-atomic mutability
}
```

### 8. Lawful Shared Atomic Structure (Valid)
```oxide
shared struct DistributedCounter {
    value: atomic<u64>
}
fn share_counter() {
    let counter = shared DistributedCounter { value: 0 };
    // Memory explicitly synced without Send/Sync magic.
    spawn(| | { counter.value.fetch_add(1, relaxed); }); 
}
```

### 9. Destructor Execution Order Checks
```oxide
fn drop_order() {
    let a = File::open("a.txt");
    let b = File::open("b.txt");
    // VALID: `b` will always drop before `a`.
}
```

### 10. The Unsafe Escape Hatch
```oxide
fn raw_escape() {
    let val = 100;
    unsafe {
        let ptr: *int = &val as *int;
        // VALID: Compiler explicitly trusts the user here.
        // The responsibility of valid access lifetime bridges falls entirely on the user.
    }
}
```
