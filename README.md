# Oxide Language Compiler

Oxide is a multi-core-first, performance-first, deterministic, C-ABI compatible systems programming language inspired by C, Go, Rust, and Zig. It features a hybrid memory model, thread-local default execution, and lock-free concurrency, entirely without Garbage Collection (GC) or implicit hidden allocations.

This project contains the **Phase 1 & Phase 2** Oxide Compiler bootstrap implementation, built in Rust to validate the language's core structural requirements.

## 🚀 Key Features

*   **Hybrid Memory Model**: Lexically scoped regional memory arenas enabling O(1) bulk deallocation (`region name { ... }`).
*   **Zero-Cost Concurrency**: Native lock-free shared structures bound to bounded default queues (`std::sync::mpsc`).
*   **Predictable Performance**: No GC, no finalizers, and strictly deterministic destructors triggered dynamically at scope boundaries (`drop(self)`).
*   **C-ABI Compatibility**: Flawless interoperability with native C functionality and POSIX system definitions.
*   **C-Transpiler Backend**: To prioritize rapid development and native target optimization without heavy LLVM dependencies in Phase 1, `oxidec` transpiles memory-safe Oxide Intermediate Representation (OxIR) directly into pure, strict C code.

---

## 🛠️ Installation & Build

Ensure you have Rust (`cargo`) installed on your system.
Additionally, you need a C compiler (`gcc` or `clang`) to compile the transpiled source code output.

```bash
# Clone the repository
git clone https://github.com/your-org/oxide.git
cd oxide/compiler

# Build the Oxide Compiler
cargo build --release
```

---

## ⚡ Quick Start: Compiling Oxide Code

The Oxide compiler (`oxidec`) supports a robust CLI to visualize the translation pipeline from parsing through transpilation.

### 1. The Demo Programs
In the `examples` directory, you will find proof-of-concept Oxide code files:
*   `examples/demo.ox` - Basic ownership, Region isolation, and mathematical type-checking.
*   `examples/demo_threads.ox` - Phase 2 multi-module verification testing `std` libraries.

### 2. Standard Compilation (`build`)
The default workflow transpiles the `.ox` source into `.c` and provides you with the final compilation command.

```bash
cargo run -- build examples/demo_threads.ox
```
*Output Process:*
1. **Lexical & Parsing Phase**: Converts source text into Abstract Syntax Trees (AST), dynamically merging any `use std::...` module imports recursively from the filesystem.
2. **Semantic Checking**: Enforces borrow checking, type safety, and Oxide's strict memory region guarantees.
3. **OxIR Generation & Validation**: The compiler maps valid AST into OxIR—a structural linear layout—and verifies it for dangling references and bounds errors.
4. **C-Transpilation**: Emits heavily optimized C representations natively patched with required forward declarations and explicit `stdatomic.h` memory ordering parameters.

### 3. Native Execution
Compile the generated `.c` file down to a native machine executable using `gcc` or `clang`:

```bash
gcc -O3 examples/demo_threads.c -o demo_threads
./demo_threads
```

---

## 🔍 Detailed Pipeline Commands
For development and debugging, you can hook into individual compiler stages:

1. **Verify Token Stream (Lexer)**
   ```bash
   cargo run -- lex examples/demo.ox
   ```

2. **Review AST (Parser)**
   ```bash
   cargo run -- parse examples/demo.ox
   ```

3. **Run Semantic Checks (Type & Memory Safety)**
   ```bash
   cargo run -- check examples/demo.ox
   ```

4. **Generate Intermediate Representation (OxIR)**
   ```bash
   cargo run -- compile examples/demo.ox
   ```

---

## 📚 Standard Library (Phase 2)
The Oxide compiler natively includes the foundational `std` library mimicking extreme lock-free safety configurations:

*   **`std::mem::slab`**: A native continuous block layout representing generic regional allocation boundaries.
*   **`std::thread`**: A zero-cost abstraction interfacing Oxide's scoped boundaries natively into POSIX (`pthread` on unix) multi-core models.
*   **`std::sync::mpsc`**: Demonstrating explicit Oxide `atomic<T>` wrappers translated correctly to `_Atomic` memory structures.

---

## 🚧 Roadmap
*   **Phase 0 (Completed):** Core compiler specifications, foundational parser (AST), and structural linear transpilation.
*   **Phase 1 (Completed):** Advanced semantics, linear data-flow validation, Region boundary enforcement (`alloc_region`), and pure C-Transpiler backend.
*   **Phase 2 (Completed):** Dynamic multi-module AST resolution (`use std::*`), Structural mocking constraints, and foundational Oxide `std` source modules.
*   **Phase 3 (Upcoming):** Full native data struct allocation memory mapping, deeper LLVM integrations natively via `inkwell`, and deeper standard file handling IO primitives. 

---

## 📜 Philosophy
Read our full system architecture mandates in `MEMORY_MODEL_v0.1.md` and `GRAMMAR_OUTLINE_v0.1.md` located in the repository root. Oxide remains committed to open governance, uncompromising velocity, and systems-level perfection.
