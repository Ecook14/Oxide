# Tour of Oxide v0.1

Oxide is a highly-deterministic, production-grade systems language optimized for multi-core compilation.

Here is a quick tour through its grammar and mechanisms.

## Structs and Variables
Variables default to immutable, requiring `mut` for assignment reassignment.

```oxide
struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

fn initialize() {
    let mut v = Vector3 { x: 1.0, y: 0.0, z: 0.0 };
    v.y = 5.5; // Requires 'mut'
}
```

## Pointers 
Oxide supports native ABI mapping with raw pointers. 
Raw pointers map explicitly to their OS equivalents.
- `*u8` = immutable raw pointer (`const uint8_t*` C equivalent)
- `*mut u8` = mutable raw pointer (`uint8_t*` C equivalent) 

```oxide
extern "c" fn memcpy(dest: *mut u8, src: *u8, n: u64);
```

## Atomics and Concurrency
Threads communicate natively through `atomic<T>` and explicitly typed memory ordering parameters mapped exactly to C11 `stdatomic.h`.

```oxide
shared struct Counter {
    pub val: atomic<u64>,
}

pub fn inc(self: Counter) {
    let v = (&self.val).load(acquire);
    (&self.val).store(v + 1, release);
}
```

## Closures
Functions can be passed via the pipe syntax `|params| { body }`. Oxide performs lexical environment capturing safely behind the scenes via dynamic, anonymous struct lifting.

```oxide
fn main() {
    let message = "Background Execution Successful";
    thread::spawn(|| {
        printf(message);
    });
}
```

## Control Flow
Loops are simplified. Oxide does not possess a traditional `for` header construct natively; `while` and `loop` serve as infinite boundaries or condition evaluations.

```oxide
let mut i = 0 as u64;
loop {
    if i == 5 as u64 { break; }
    i = i + 1;
}
```
