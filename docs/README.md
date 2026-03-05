# Oxide Documentation

Welcome to the documentation for **Oxide Version 0.1**, a multi-core-first, hybrid-memory, deterministic systems language.

## Language Foundations
- [Development Roadmap & v0.1 Feature Freeze](ROADMAP.md): Detailed tracks covering Syntax, Types, Semantics, Stdlib, Tooling, and ABI.
- [v0.1 Formal Specification](V0_1_SPEC.md): Formal definition of the v0.1 Region model, Destructors, ADTs, and FFI rules.
- [Tour of Oxide](language_tour.md): A crash course in the syntax, types, and semantics of Oxide.
- [Memory Model](../MEMORY_MODEL_v0.1.md): Deep dive into Oxide's region-based allocation, arenas, and deterministic destructors without Garbage Collection.
- [OxIR Specification](../OXIR_SPEC_v0.1.md): Documentation on the Oxide Intermediate Representation (OxIR) architecture.

## Standard Library (v0.1)
The standard library is designed to offer zero-cost, high-performance concurrency and memory primitives.

### Memory (`std::mem`)
- [`slab::SlabAllocator`](std_mem_slab.md): A bump-pointer, highly aligned, region-scoped slab allocator.
- `arena` (Planned): Multi-region hierarchical fast paths.

### Concurrency (`std::sync`)
- [`mpsc::MpscQueue`](std_sync_mpsc.md): A lock-free, bounded multiple-producer, single-consumer ring buffer leveraging sequence-based atomic synchronization.
- `atomic`: Strictly C11-compatible atomic intrinsics leveraging `memory_order` explicit parameters.

### Execution (`std::thread`)
- [`thread::spawn`](std_thread.md): Zero-cost native OS thread bindings combined with dynamic closure dispatching and capture encapsulation.
