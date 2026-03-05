use std::mem::arena;

extern "c" fn printf(fmt: *u8, arg1: u64) -> i32;

fn test_basic_alloc() {
    printf("--- Basic Alloc ---\n" as *u8, 0 as u64);
    
    // 1. Initialize arena with a chunk size of 128 bytes
    let arena_ctx = arena::new_arena(128 as usize);
    
    // 2. Perform aligned allocations
    let p1 = arena::alloc(arena_ctx, 32 as usize, 8 as usize);
    let p2 = arena::alloc(arena_ctx, 16 as usize, 8 as usize); // 48 bytes used
    let p3 = arena::alloc(arena_ctx, 64 as usize, 16 as usize); // 16 alignment padding pushes it
    
    // 3. Perform an oversized allocation (triggers chunk list append)
    let p_big = arena::alloc(arena_ctx, 1024 as usize, 8 as usize);
    
    // 4. Reset arena (retains heap blocks but rewinds used counter)
    arena::reset(arena_ctx);
    
    // 5. Re-allocate (Should land exactly on p1)
    let p1_new = arena::alloc(arena_ctx, 32 as usize, 8 as usize);
    
    if p1 == p1_new {
        printf("  Reset verification: PASSED\n" as *u8, 0 as u64);
    } else {
        printf("  Reset verification: FAILED\n" as *u8, 0 as u64);
    }
    
    // 6. Hard free all blocks
    arena::destroy(arena_ctx);
    printf("  Arena Drop: COMPLETED\n" as *u8, 0 as u64);
}

fn main() -> i64 {
    printf("Starting Arena Tests...\n" as *u8, 0 as u64);
    
    test_basic_alloc();
    
    printf("All Tests Completed Successfully.\n" as *u8, 0 as u64);
    return 0 as i64;
}
