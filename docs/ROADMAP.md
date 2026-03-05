# Oxide Language Development Roadmap & v0.1 Feature Freeze

Oxide is being constructed explicitly as an **Infrastructure-grade, proxy/server-oriented, deterministic systems language**. It is designed to act as a modern replacement for C in multi-core, high-throughput, latency-critical networking environments. 

To achieve this, Oxide sacrifices hidden allocations, garbage collection, and runtime magic in favor of region-first memory, explicit layouts, and zero-cost safety.

---

## 🔒 v0.1 Feature Freeze Contract

Version 0.1 of Oxide provides foundational stability. The **v0.1 Freeze** guarantees that the following elements are architecturally locked. Breaking changes to these systems will not occur without a major version bump:

1. **Syntax Stability**: No breaking parser changes to core control flow, structural bindings, closures, or existing operators (outside of critical bug fixes).
2. **Deterministic Destructors**: The drop semantics, order of destruction, and region boundary bulk-free mechanics are strictly frozen.
3. **Region Strictness**: The `alloc_region` ownership and linear data-flow rules are logically locked.
4. **Panic Strategy (`abort`-only)**: Unwinding is completely omitted in v0.1. A panic triggers an immediate OS trap/abort, ensuring zero hidden complexity with region destructors.
5. **Atomic Memory Model**: Sequence-consistent atomics are the default and are structurally frozen.
6. **Result/Option Standard**: The structural representation and parsing of explicit `Result`/`Option` ADTs (along with the `?` error propagation operator) are locked to provide deterministic failure paths.

---

## Development Tracks

### Track 1: Syntax & Grammar (`SYNTAX`)
Focuses on the lexical and parsing frontend.
- **Phase 1 (v0.1 - Feature Freeze):** Basic control flow (`if`, `while`, `loop`), primitive bindings (`let`), struct instantiation, closures (`||`), and early error propagation (`?` operator).
- **Phase 2 (v0.2):** Pattern matching (`match` statements), explicit module parsing (`mod`, `use path::*`).
- **Phase 3 (v0.3):** Simple generics syntax (`fn foo<T>()`) and visibility modifiers (`pub(crate)`).

### Track 2: Type System & Analyzer (`TYPES`)
Focuses on strict safety, C-ABI integrity, and explicit memory representation.
- **Phase 1 (v0.1 - Feature Freeze):** 
  - Primitives (`u8`, `usize`), raw pointers (`*mut u8`).
  - Generic struct evaluation & zero-cost ownership tracking.
  - **Enums (Algebraic Data Types)** and explicit **`Result` / `Option`** types.
  - **Memory Layout Introspection** (`size_of<T>()`, `align_of<T>()`, `offset_of<T>()`) to guarantee safe packet/socket buffer mapping.
- **Phase 2 (v0.2):** Union layouts, advanced escape analysis.
- **Phase 3 (v0.3):** Const-generics for arrays (e.g., `[u8; 1024]`), and zero-sized types (ZST).

### Track 3: Core Semantics & OxIR (`SEMANTICS`)
Focuses on intermediate lowering, optimizations, and memory-model validation.
- **Phase 1 (v0.1 - Feature Freeze):** 
  - Linear data-flow validation (prevent use-after-move).
  - Region boundary enforcement.
  - Safe FFI structural mapping (`StoreField`, `CallVoid`).
  - Sequence-consistent `<stdatomic.h>` mappings.
  - **Deterministic Destructors** (exact drop invocation boundaries).
  - **Basic Dead-Code Elimination (DCE)** IR optimization.
- **Phase 2 (v0.2):** Escape analysis across explicit region boundaries.
- **Phase 3 (v0.3):** Advanced OxIR optimizations (Constant Folding) and thread-local core pinning rules.

### Track 4: Standard Library (`STDLIB`)
Focuses on zero-dependency, lock-free, and high-performance primitives intended for infrastructure.
- **Phase 1 (v0.1 - Feature Freeze):** 
  - `std::mem::slab` (Fixed-size block allocator).
  - `std::mem::arena` (Multi-region hierarchical allocator).
  - `std::sync::mpsc` (Lock-free queues).
  - `std::thread` (Zero-cost native OS threads & closures). *Note: This provides primitive OS thread wrappers only. Event-driven architectures will be evaluated post-v0.1.*
  - **`std::io` Minimal Abstraction** (Reader/Writer trait-like structs) to unify protocol streams.
- **Phase 2 (v0.2):** `std::fs` (File system abstractions), `std::net` (Socket multiplexing), and `std::collections` (Sharded HashMap).
- **Phase 3 (v0.3):** Backpressure-aware bounded channels and explicit thread pinning wrappers.

### Track 5: Tooling & Build System (`TOOLING`)
Focuses on developer experience and internal visibility.
- **Phase 1 (v0.1 - Feature Freeze):** 
  - Basic CLI compiler (`oxidec lex/parse/check/build/run`).
  - IR visibility (`oxidec --emit-oxir`, `--emit-llvm`, `--emit-asm`) to prevent blind compiler development.
- **Phase 2:** Oxide Package Manager (`ox`), built-in formatter (`ox fmt`), and Test runner (`ox test`).

### Track 6: ABI & Interoperability (`ABI`)
Focuses on low-level binary stability and cross-language boundaries.
- **Phase 1 (v0.1 - Feature Freeze):** Stable C ABI mapping, layout tests, and `repr(C)`-equivalent guarantees.
- **Phase 2 (v0.2):** Deterministic panic boundaries (C-FFI unwinds strictly trap), cross-language static linking tests.
- **Phase 3 (v0.3):** Shared object (`.so` / `.dll`) support, aggressive cross-language link-time optimization (LTO).
