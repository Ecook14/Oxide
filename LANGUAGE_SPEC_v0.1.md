# Oxide Language Specification v0.1 (Phase 0 Draft)

> **Status**: Phase 0 Draft  
> **Date**: 2026-02-27  
> **Authority**: Language Architecture Team

This document consolidates the foundational Oxide specifications into a single authoritative reference. 
Individual specification documents (`MEMORY_MODEL_v0.1.md`, `ATOMIC_SEMANTICS_v0.1.md`, `OXIR_SPEC_v0.1.md`, `GRAMMAR_OUTLINE_v0.1.md`) remain canonical for their respective domains. This document provides ordering, cross-referencing, and an executive summary.

---

## Table of Contents
1. [Language Identity](#1-language-identity)
2. [Memory Model & Ownership](#2-memory-model--ownership)
3. [Atomic & Shared Memory Semantics](#3-atomic--shared-memory-semantics)
4. [Grammar & Syntax](#4-grammar--syntax)
5. [OxIR Intermediate Representation](#5-oxir-intermediate-representation)
6. [Undefined Behavior Catalog](#6-undefined-behavior-catalog)
7. [Cross-Reference Matrix](#7-cross-reference-matrix)

---

## 1. Language Identity

Oxide is a multi-core-first, performance-first, deterministic systems language with:
- **No garbage collector**
- **No lifetime annotations** — ownership is structural and local
- **No implicit global state** — all sharing is explicit
- **Full C ABI compatibility** — `extern "c"` FFI
- **Hybrid memory** — Regions (arenas) + deterministic destructors
- **Hybrid concurrency** — Message passing + explicit `shared` memory

---

## 2. Memory Model & Ownership

*Full specification: [MEMORY_MODEL_v0.1.md](./MEMORY_MODEL_v0.1.md)*

### Summary of Guarantees
| Rule | Guarantee |
|------|-----------|
| Single Ownership | Every value has exactly one owner |
| Move Semantics | Assignment transfers ownership; source becomes invalid |
| Lexical Borrowing | `&T` (immutable) or `&mut T` (exclusive mutable), no lifetime annotations |
| Regions | Lexically scoped arenas with O(1) bulk free, no per-object destructors |
| Region Escape | Compiler prevents values from escaping their region statically |
| Destructors | Reverse construction order, no panic, no implicit alloc |
| Thread-Local Default | All data thread-local unless marked `shared` |

### Key Constraints for Compiler
- Borrow validity resolved **lexically**, not globally.
- Mutable aliasing forbidden: `&mut T` is exclusive.
- Region-allocated values bypass destructors.

---

## 3. Atomic & Shared Memory Semantics

*Full specification: [ATOMIC_SEMANTICS_v0.1.md](./ATOMIC_SEMANTICS_v0.1.md)*

### Summary of Guarantees
| Element | Rule |
|---------|------|
| `shared struct` | Only `atomic<T>`, immutable fields, or `unsafe` blocks |
| `atomic<T>` | First-class primitive, explicit ordering required on every operation |
| Memory Orderings | `relaxed`, `acquire`, `release`, `acq_rel`, `seq_cst` (C11 compatible) |
| No Default | Forgetting to specify ordering is a compile-time error |
| No Send/Sync | Sharing is structural, not trait-based |

---

## 4. Grammar & Syntax

*Full specification: [GRAMMAR_OUTLINE_v0.1.md](./GRAMMAR_OUTLINE_v0.1.md)*

### Keyword Set (22 keywords)
`fn`, `struct`, `enum`, `let`, `mut`, `if`, `else`, `while`, `for`, `loop`, `break`, `continue`, `return`, `match`, `in`, `region`, `shared`, `atomic`, `unsafe`, `drop`, `use`, `pub`, `mod`, `extern`

### Type System
- Integers: `u8`–`u64`, `i8`–`i64`, `usize`, `isize`
- Floats: `f32`, `f64`
- `bool`, `char`
- `atomic<T>`, `&T`, `&mut T`, `*T`, `[T; N]`

### Syntax Principles
- No macros (v1), no operator overloading, no implicit conversions
- Parentheses around `if`/`while` conditions optional
- Pattern matching must be exhaustive

---

## 5. OxIR Intermediate Representation

*Full specification: [OXIR_SPEC_v0.1.md](./OXIR_SPEC_v0.1.md)*

### Summary
OxIR is an SSA-based intermediate representation that preserves:
- Ownership transfer semantics (`move`)
- Region boundaries (`alloc_region`, `region_bulk_free`)
- Borrow scopes (`borrow_immut`, `borrow_mut`, `end_borrow`)
- Atomic orderings (`atomic_load`, `atomic_store`, `atomic_rmw`, `atomic_cmpxchg`)

**Lowering to LLVM never erases these boundaries.**

---

## 6. Undefined Behavior Catalog

| # | UB Condition |
|---|-------------|
| 1 | Dereferencing invalid, null, or unaligned pointer |
| 2 | Data race on non-atomic memory |
| 3 | Escaping region-bound value (via `unsafe`) |
| 4 | Double-free outside region model |
| 5 | Violating atomic ordering contract |
| 6 | Mixing atomic/non-atomic access on same address |

---

## 7. Cross-Reference Matrix

| Semantic Domain | Defined In | Consumed By |
|-----------------|-----------|-------------|
| Ownership & Regions | MEMORY_MODEL | Parser, Type Checker, OxIR Gen |
| Borrow Rules | MEMORY_MODEL | Type Checker, OxIR Validator |
| Atomic Orderings | ATOMIC_SEMANTICS | OxIR Gen, LLVM Lowering |
| Shared Struct Rules | ATOMIC_SEMANTICS | Type Checker |
| Token Definitions | GRAMMAR_OUTLINE | Lexer (`token.rs`) |
| AST Structure | GRAMMAR_OUTLINE | Parser (`ast.rs`) |
| IR Instructions | OXIR_SPEC | OxIR Gen (Phase 1) |
