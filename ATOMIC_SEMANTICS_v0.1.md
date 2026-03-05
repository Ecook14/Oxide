# Oxide Atomic & Shared Memory Semantics v0.1

This document formally defines Oxide's concurrency primitives, specifically focusing on the `shared` structural boundary and the hardware-level contracts of `atomic<T>` operations. This specification guarantees deterministic multi-core scaling while providing 1:1 C11 memory model ABI compatibility.

---

## I. The `shared` Structural Boundary

By default, **all data in Oxide is thread-local**. 
No struct, array, region, or primitive can be accessed across thread boundaries unless it is structurally marked as `shared`.

### 1. Structural Definition
To share memory, the structure must be explicitly defined using the `shared` keyword:
```oxide
shared struct CacheLine {
    version: atomic<u64>,
    data: [u8; 64],
}
```

### 2. Constraints on `shared` Types
The compiler enforces strict rules on the composition of a `shared struct`:
1. **Atomics**: Fields may be `atomic<T>` (where `T` is a primitive integer, boolean, or raw pointer).
2. **Immutable Data**: Fields may be deeply immutable (read-only after initialization).
3. **No Unprotected Mutability**: Standard mutable fields (`mut` or implicit `mut`) are strictly forbidden. Modifying shared data requires either an atomic operation or an `unsafe { ... }` block.
4. **No Implicit Traits**: Oxide does not implement magical `Send` or `Sync` marker traits. A type is shareable if and only if its structural definition adheres to the `shared` rules.

### 3. Region Sharing
Regions are thread-local by default. However, a region can be marked shared to allow cross-core access to its bulk allocations:
```oxide
shared region R1 { ... }
```
Objects allocated within a `shared region` must still adhere to the `shared struct` composition rules.

---

## II. First-Class `atomic<T>` Primitives

Oxide treats atomics as first-class generic primitives: `atomic<bool>`, `atomic<int>`, `atomic<u64>`, `atomic<*T>`.

### 1. Operations
Every atomic manipulation in safe code must explicitly declare its memory ordering. There is no implicit default (e.g., defaulting to `seq_cst` is forbidden to prevent hidden performance cliffs).
- `load(order)`
- `store(val, order)`
- `swap(val, order)`
- `compare_exchange(expected, new, success_order, failure_order)`
- `fetch_add(val, order)`
- `fetch_sub(val, order)`
- `fetch_and(val, order)`
- `fetch_or(val, order)`
- `fetch_xor(val, order)`

---

## III. Formal Memory Ordering Contracts (C11 Equivalent)

Oxide enforces the 5 explicit memory ordering guarantees. These map identically to the C11 / LLVM atomics model to ensure pure ABI compatibility.

### 1. `relaxed`
- **Definition**: Guarantees atomicity of the operation on the specific variable only.
- **Constraints**: 
  - NO synchronization or ordering guarantees across threads for *other* variables.
  - Vulnerable to instruction reordering by the compiler and CPU.
- **Use Case**: Counters, stats, UUID generation where exact synchronization of surrounding memory is irrelevant.
```oxide
let count = shared Counter { c: 0 };
count.c.fetch_add(1, relaxed);
```

### 2. `acquire`
- **Definition**: When applied to a `load` operation, ensures that no memory reads or writes in the current thread can be reordered *before* this load.
- **Pairs With**: `release` stores in other threads.
- **Use Case**: Reading a lock, acquiring a spinlock, consuming a published pointer.
```oxide
while (lock.flag.load(acquire) == false) { yield(); }
```

### 3. `release`
- **Definition**: When applied to a `store` operation, ensures that no memory reads or writes in the current thread can be reordered *after* this store.
- **Pairs With**: `acquire` loads in other threads.
- **Use Case**: Releasing a lock, publishing data to a lock-free queue, finalizing a region.
```oxide
lock.flag.store(true, release);
```

### 4. `acq_rel` (Acquire-Release)
- **Definition**: Used only on Read-Modify-Write (RMW) operations (e.g., `swap`, `fetch_add`, `compare_exchange`). Matches both `acquire` and `release` semantics simultaneously.
- **Use Case**: Implementing thread-safe reference counting (if built inside a library), or complex lock-free data structures like skip lists.
```oxide
let old = node.next.swap(new_ptr, acq_rel);
```

### 5. `seq_cst` (Sequentially Consistent)
- **Definition**: The strongest ordering. Provides `acquire` semantics for loads, `release` semantics for stores, and additionally guarantees a single total global order of all `seq_cst` operations across all threads.
- **Use Case**: Initializing complex distributed synchronization states where absolute global agreement is required.
- **Performance Note**: Triggers full CPU memory barriers. The Performance Team strictly audits the use of `seq_cst`.

---

## IV. Lock-Free Guarantees & OS Interaction

1. **Wait-Free Integers**: On x86_64 and AArch64, `atomic<u8>` through `atomic<u64>` must map to wait-free hardware instructions.
2. **Atomic Alignment**: The compiler must strictly align atomic primitives to their natural size boundary (e.g., 8-byte alignment for `atomic<u64>`). Misalignment of atomics is considered Undefined Behavior (UB), which is caught by the compiler statically within safe Oxide code.
3. **Compare-Exchange Loops**: Oxide's `compare_exchange` behaves as the \"strong\" variant. A \"weak\" variant (`compare_exchange_weak`) is provided exclusively for optimization loops on ARM (LL/SC architectures).

---

## V. Undefined Behavior (UB) in Concurrency

The following actions result in absolute Undefined Behavior and are the strict responsibility of the programmer when using `unsafe`:
1. Performing a raw non-atomic write to an immutable field inside a `shared struct` via pointer casting.
2. Mixing `atomic<T>` operations with non-atomic raw writes on the exact same underlying memory address.
3. Attempting to pass a non-`shared` allocation reference across a thread boundary using inline assembly or C-FFI.

---

## VI. OxIR Representation

In OxIR, concurrency instructions retain their strict semantics:
```text
%1 = atomic_load [acquire] %ptr
%2 = atomic_cas [acq_rel, relaxed] %ptr, %expected, %new
```
Lowering these to LLVM IR maps directly to `load atomic acquire` and `cmpxchg acq_rel relaxed`. The compiler will NEVER upgrade or downgrade user-specified orderings.

---

## VII. Reference Examples: Valid vs. Invalid 

### 1. Correct Acquire/Release Handshake
```oxide
shared struct Message {
    ready: atomic<bool>,
    data: int, // Immutable once published
}

fn producer(msg: &mut shared Message) {
    // raw setup allowed before sharing conceptually (unsafe boundary or initialization)
    unsafe { write_unprotected(&msg.data, 42); } 
    msg.ready.store(true, release);
}

fn consumer(msg: &shared Message) {
    while (!msg.ready.load(acquire)) { yield(); }
    // `acquire` guarantees we read the updated `data`
    print(msg.data); 
}
```

### 2. Forbidden Implicit Orderings (Blocked)
```oxide
shared struct Bad {
    flag: atomic<bool>
}
fn act(b: &shared Bad) {
    // ERROR: Must specify ordering (e.g., relaxed, seq_cst)
    b.flag.store(true); 
}
```

### 3. Spinlock Primitive Verification
```oxide
shared struct SpinLock {
    locked: atomic<bool>
}

// Validation of correct atomic loop mapping
fn lock(l: &shared SpinLock) {
    while l.locked.compare_exchange(false, true, acquire, relaxed) == false {
        // CPU yield instruction
        intrinsic::pause(); 
    }
}

fn unlock(l: &shared SpinLock) {
    l.locked.store(false, release);
}
```
