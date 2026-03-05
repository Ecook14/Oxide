# Oxide Language Grammar Outline v0.1

This document outlines the base grammar for Oxide, serving as the blueprint for the parser prototype. The syntax philosophy strictly enforces:
- Minimal keywords
- No macros in v1
- No operator overloading
- No implicit conversions
- No hidden allocations

---

## I. Lexical Structure

### 1. Keywords
Oxide uses a tightly constrained set of keywords.

**Core:**
- `fn`, `struct`, `enum`, `let`, `mut`
- `if`, `else`, `while`, `for`, `loop`, `break`, `continue`, `return`
- `match`, `=>` 

**Memory & Concurrency:**
- `region`, `shared`, `atomic`
- `unsafe`, `drop`

**Modules:**
- `use`, `pub`, `mod`

*(Keywords like `class`, `yield` (generator), `async`, `await` are intentionally excluded for Phase 0).*

### 2. Operators
Mathematical and logical operators map directly to standard C-like semantics:
- `+`, `-`, `*`, `/`, `%`
- `==`, `!=`, `<`, `>`, `<=`, `>=`
- `&&`, `||`, `!`
- Bitwise: `&`, `|`, `^`, `<<`, `>>`, `~`
- Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `&=`, `|=`, `^=`, `<<=`, `>>=`

No user-defined operator overloading is permitted. No spaceship operator (`<=>`). 

### 3. Types
All types must be explicitly sized. No implicit fallback types (e.g., `int` without size).

- **Integers**: `u8`, `u16`, `u32`, `u64`, `usize`, `i8`, `i16`, `i32`, `i64`, `isize`
- **Floats**: `f32`, `f64`
- **Booleans**: `bool` (values: `true`, `false`)
- **Chars**: `char` (32-bit Unicode scalar)
- **Atomics**: `atomic<T>`
- **Pointers**: `*u8`, `*mut T` (only usable in `unsafe`)
- **References**: `&T`, `&mut T`

---

## II. Module & File Structure

An Oxide source file consists of a sequence of top-level declarations:
1. `use` statements
2. `mod` declarations
3. Type definitions (`struct`, `enum`)
4. Functions (`fn`)

```oxide
use std::mem::region;
use std::sync::atomic;

pub mod net;
```

---

## III. Functions

Functions enforce explicit signatures. No return type deduction for public APIs. 
Return type is `void` implicitly if omitted (`-> ()` is allowed but not strictly required).

```oxide
fn add(a: u32, b: u32) -> u32 {
    return a + b;
}

// C-ABI interop is explicit
extern "c" fn read(fd: i32, buf: *u8, size: usize) -> isize;
```

---

## IV. Data Structures

### 1. Structs
Struct fields are private by default. 

```oxide
struct Point {
    pub x: f32,
    pub y: f32,
}

// The structural shared boundary must explicitly wrap the entire struct definition
shared struct AtomicCounter {
    pub count: atomic<u64>,
}
```

### 2. Enums
Oxide enums support tagged unions.

```oxide
enum Result {
    Ok(u32),
    Err(i32),
}
```

---

## V. Variable Declarations & Mutability

Variables are immutable by default. Mutability requires the `mut` keyword.

```oxide
let x: u32 = 10;
let mut y = 20; // Type inference only allowed for local variables
y = 30;
```

---

## VI. Control Flow

### 1. If / Else
Standard conditional blocks. Parentheses around conditions are optional but discouraged.

```oxide
if x > 10 {
    print(1);
} else if x == 5 {
    print(2);
} else {
    print(3);
}
```

### 2. Loops
Oxide provides `loop` (infinite), `while`, and `for` (iterators).

```oxide
loop {
    if !ready() { break; }
}

while count > 0 {
    count -= 1;
}

for item in collection {
    process(item);
}
```

### 3. Match Expressions
Required for safe tag extraction from enums. Pattern matching must be exhaustive.

```oxide
match res {
    Ok(val) => print(val),
    Err(e) => handle_error(e),
}
```

---

## VII. Memory Scopes & Regions

### 1. Region Blocks
Regions define explicit scopes for the lexically scoped arena allocator.

```oxide
fn process() {
    region r {
        let buf = r.alloc_array(1024);
        read_data(buf);
    } // buf is instantly freed here
}
```

### 2. Unsafe Blocks
Required to execute pointer arithmetic and manual memory casting.

```oxide
unsafe {
    let raw = &val as *mut u32;
    *raw = 100;
}
```

---

## VIII. Comments

Standard C-style comments:
- Single line: `// Comment`
- Block: `/* Comment */`
- Doc comments: `/// Public API Document`
