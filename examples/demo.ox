// ============================================================
// demo.ox — Oxide Language Demo
// ============================================================
// This file exercises every grammar construct defined in
// GRAMMAR_OUTLINE_v0.1.md for parser validation.
// ============================================================

use std::sync::atomic;
use std::mem::region;

pub mod net;

// ── Struct definitions ──

struct Point {
    pub x: f32,
    pub y: f32
}

shared struct AtomicCounter {
    pub count: atomic<u64>
}

// ── Enum ──

enum Result {
    Ok(i64),
    Err(i64)
}

// ── C ABI interop ──

extern "c" fn read(fd: i32, buf: *u8, size: usize) -> isize;
extern "c" fn printf(fmt: &u8, val: i64) -> i32;

// ── Functions ──

fn add(a: i64, b: i64) -> i64 {
    return a + b;
}

fn demo_control_flow() {
    let x: i64 = 10;
    let mut y = 20;

    if x > 10 {
        y = 1;
    } else if x == 5 {
        y = 2;
    } else {
        y = 3;
    }

    while y > 0 {
        y -= 1;
    }

    loop {
        if y == 0 {
            break;
        }
        continue;
    }
}

fn demo_region() {
    region r {
        let buf = 1024;
    }
}

fn demo_unsafe() {
    let val = 100;
    unsafe {
        let raw = 42;
    }
}

fn demo_match() {
    let res = 10;
    match res {
        Ok(_) => 1,
        Err(_) => 0
    }
}

fn main() {
    let result = add(10, 20);
    demo_control_flow();
    demo_region();
    demo_unsafe();
    printf("Oxide Demo Completed Successfully! Add result: %ld\n", result);
}
