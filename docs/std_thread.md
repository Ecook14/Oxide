# Oxide Standard Library: `std::thread`

The `thread` module implements the foundation of Oxide's **Multi-Core First** directive. It provides zero-cost abstractions binding Oxide closure execution blocks to C-ABI POSIX native system threads safely.

## Mechanisms

### `spawn`
```oxide
pub fn spawn(f: |()| -> ()) -> Thread
```
`spawn` accepts a parameter-less closure. Behind the scenes:
1. Oxide's front-end **captures** lexical variables, generating an anonymous environment struct (`_clos_env_X`).
2. The OxIR translates the logic into a standalone *trampoline* `_clos_tramp_X`, resolving the structural indirection safely.
3. Finally, `pthread_create` initializes a bare-metal OS thread natively bound to the trampoline, pushing the memory context directly via C void-pointer boundary passing.

If `pthread_create` indicates failure, the library panics predictably rather than returning undefined behavior.

### Handling References (`Thread` struct)
Wait constructs to intercept execution completion natively utilizing `pthread_join(t.handle)` are currently integrated exclusively through the raw standard library. Higher-order Futures mapping is documented in later framework specifications.
