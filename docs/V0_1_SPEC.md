# Oxide v0.1 Formal Specification

This document defines the formal operational semantics, layout guarantees, and system constraints for Oxide **v0.1**. While `ROADMAP.md` dictates the direction of the language, this spec defines the law. Any compiler implementation of Oxide v0.1 MUST adhere to these exact constraints.

---

## 1. Region Memory Model

Oxide strictly distinguishes between *stack* allocation and *region* allocation. The compiler guarantees that no implicit dynamic memory allocation (heap) is ever inserted.

### 1.1 Region Boundaries
- A region is activated via `alloc_region %region_name`.
- Regions are **lexically scoped**.
- All pointers bound to a region are strictly invalid the moment the lexical scope of the region ends.

### 1.2 Cross-Region Pointers
- A region-allocated object **may not** contain pointers to memory in a shorter-lived (narrower) region.
- Stack variables **may** store pointers to region memory, but the compiler statically ensures (escape analysis) that these stack variables do not outlive the region's lexical scope. Escape analysis violations result in a compile-time error.

### 1.2 Bulk-Free Guarantee
- Memory allocated within a region MUST be deallocated in strictly **$O(1)$ time** via a bulk-free mechanism.
- Individual allocations within a region cannot be freed individually.

### 1.3 Memory Layout Introspection
In v0.1, the compiler provides built-in intrinsic properties guaranteeing introspection statically:
- `size_of<T>()`: Evaluates to the byte-size of the type at compile time.
- `align_of<T>()`: Evaluates to the byte-alignment required by the target OS architecture.
- `offset_of<T>(field)`: Evaluates to the exact offset of a property from the start of the struct.

---

## 2. Deterministic Destructors (`drop`)

Oxide guarantees predictable resource finalization without runtime collection. 

### 2.1 Scope of Destructors
- Destructors (`drop_in_place()`) execute deterministically at the exact boundary of ownership expiration.
- For **stack-allocated** types: Destructors run exactly at the end of the enclosing block scope.
- For **region-allocated** types: Destructors run strictly immediately **before** the `region_bulk_free` of their parent region.

### 2.2 Execution Order
- Destructors are guaranteed to execute in **strict Last-In, First-Out (LIFO)** block execution order. The most recently allocated entity in a scope is destroyed first.
- **Nested Regions:** Nested regions are destroyed in strict lexical reverse order, fully executing destructor chains before parent region teardown.
- Explicit invocation of a custom `drop(self)` method consumes ownership and bypasses the implicit end-of-scope compiler destructor, transferring lifecycle control entirely to the developer.

### 2.3 Double-Drop Prevention
- Invoking `drop(self)` multiple times on the same owned instance is strictly a **compile-time error**, enforced natively by the linear data-flow ownership tracker.

---

## 3. Algebraic Data Types (Enums)

Enums in Oxide v0.1 are true Algebraic Data Types (ADTs), capable of storing state natively inside variants. 

### 3.1 Tagged Union Layout
To ensure memory predictability, enums are lowered into a standardized tagged union layout mirroring C equivalents.
- The **Tag** is an unsigned integer escalating deterministically by variant count:
  - $\le 255$ variants $\rightarrow$ `u8`
  - $\le 65,535$ variants $\rightarrow$ `u16`
  - Else $\rightarrow$ `u32`
- The **Payload** is a contiguous C-compatible `union` overlapping identically to the largest variant's size.
- The struct is packed according to standard `repr(C)` padding rules to align the payload naturally.

Example logic C-ABI expansion:
```c
struct MyEnum {
    uint8_t tag;
    union {
        uint64_t variant_a;
        struct { uint32_t x; uint32_t y; } variant_b;
    } payload;
};
```

---

## 4. `Result` and `Option` Representation

All fallible logic and optional data MUST be structured through `Result<T, E>` and `Option<T>`. 

### 4.1 Structural Completeness
- Neither `Result` nor `Option` are inherently "magic" language concepts; they are rigidly lowered as standard ADT `Enums` (Section 3).
- `Option<T>` maps to: Tag 0 (`None`), Tag 1 (`Some(T)`).
- `Result<T, E>` maps to: Tag 0 (`Ok(T)`), Tag 1 (`Err(E)`).
- The `?` operator is strictly syntactic sugar that expands to a conditional branch returning the `Err(E)` variant out of the current stack frame.

### 4.2 Prohibited Niche Optimization
- In v0.1, the compiler is **explicitly forbidden** from performing niche-filling optimizations (e.g., packing `Option<&T>` to a null pointer). Tags and payloads are rigidly laid out to ensure maximum predictable memory representation for protocol layout stability.

---

## 5. Panic Semantics

Oxide natively rejects the concept of exception unwinding. Unwinding injects hidden control flow and invalidates rigid region cleanup invariants.

### 5.1 The `abort`-Only Contract
- A `panic!()` in Oxide strictly evaluates to a fast process trap (`abort()`).
- There is **no stack unwinding**.
- There is **no `catch` logic**.
- When an assert fails or a panic is invoked, the OS halts the process instantly. Developers must handle expected failures logically via `Result<T, E>`.

---

## 6. FFI Boundary & C-ABI

Oxide heavily prioritizes latency-critical integration with existing infra systems. Oxide v0.1 guarantees C-interoperability without "glue" tax.

### 6.1 `repr(C)` Equivalence
- Oxide primitive integers (`u8`, `i32`, `usize`, etc.) and basic pointers directly map 1:1 to C99 standard library intrinsic counterparts (`uint8_t`, `int32_t`, `uintptr_t`).
- Oxide structs without special generic bounds are lowered deterministically into `struct` definitions laid out identically to the target platform's **System V / MS ABI specification**. 

### 6.2 Padding Guarantees
- Padding bytes introduced for alignment are **uninitialized** unless explicitly zeroed by the user.
- The compiler **may not** reorder fields. Memory order rigidly mirrors lexical declaration order (`repr(C)` equivalence).

### 6.2 The Void Intercept
- FFI functions defined as `extern "c"` returning `void` (e.g. `free`, `abort`) evaluate internally via the `OxIR::CallVoid` instruction. 
- The Oxide code generator explicitly bypasses return-variable assignments natively, satisfying C-compiler void-expression constraints without utilizing dummy storage registers.

### 6.4 Trap across Boundaries
Because Oxide does not unwind (Section 5), it is statically safe for C functions to invoke Oxide callbacks. If the Oxide callback panics, it simply traps the entire C host application identically to a native segmentation fault.

---

## 7. Undefined Behavior & Compiler Obligations

As a systems language, Oxide mandates strong definitions around operational constraints.

### 7.1 Compile-Time Errors
The compiler is legally obligated to reject the following statically:
- **Use-After-Region-End**: Referencing pointers tied to a consumed or out-of-scope region.
- **Double Drop**: Invoking destruction methods explicitly multiple times on the same object.
- **Cross-Region Escape**: Retaining a pointer in a long-lived region that references data in a short-lived region.
- **Dangling Borrows**: Failing to balance `borrow_mut` or `borrow_immut` markers before a value is moved or a function returns.

### 7.2 Undefined Behavior (UB)
Oxide relies on its strong semantics to prevent invalid states, but because `unsafe` is provided as an escape hatch, developers performing manual memory management or raw pointer math expose themselves to the following Undefined Behaviors:
- **Data Races**: Concurrent data modification without synchronized atomic memory barriers or explicit lock primitives.
- **Out-of-Bounds Access**: Pointer arithmetic that reads past the extent of the allocated underlying array or struct boundary.
- **Manual Alias Violation**: Creating multiple mutable aliases (`&mut T`) to the same data concurrently via unsafe casts.
