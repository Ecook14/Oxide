// ============================================================
// examples/demo_threads.ox
// Verification file for Phase 2 implementation specs.
// ============================================================

use std::thread;
use std::sync::mpsc;
use std::mem::slab;

extern "c" fn printf(format: &u8, val: u64) -> i32;
extern "c" fn malloc(size: u64) -> u64;

fn dummy_thread(arg: u64) -> u64 {
    let msg = "Hello from Oxide spawned thread!\n";
    printf(msg, 0 as u64);
    return 0 as u64;
}

fn test_library_resolution() -> i64 {
    // 1. Resolve custom allocation via std/mem/slab
    let allocator = slab::create_slab(4096 as usize);
    
    // 2. Resolve structural sharing model via std/sync/mpsc
    let queue = mpsc::new_queue(100 as usize);
    
    // 3. Resolve high-level closure-based thread creation
    let handle = thread::spawn(|| {
        printf("Hello from Oxide spawned thread via closure!\n", 0 as u64);
    });
    
    return allocator.max_size as i64;
}

fn main() -> i64 {
    let result = test_library_resolution();
    let msg = "Oxide Phase 2 Standard Library Compilation Success! Slab capacity: %ld\n";
    printf(msg, result as u64);
    
    return 0;
}
