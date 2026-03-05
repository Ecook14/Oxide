use std::mem::slab;

extern "c" fn printf(fmt: &u8, p1: usize) -> i32;

fn main() {
    // Create a 1MB slab
    let slab = slab::create_slab((1024 * 1024) as usize);
    
    // Allocate some blocks
    let p1 = slab::alloc(slab, 100 as usize, 8 as usize);
    let p2 = slab::alloc(slab, 200 as usize, 16 as usize);
    let p3 = slab::alloc(slab, 50 as usize, 8 as usize);
    
    printf("Slab allocations:\n", 0 as usize);
    printf("  p1: %p\n", p1);
    printf("  p2: %p\n", p2);
    printf("  p3: %p\n", p3);
    
    // Verify offsets
    if p2 >= p1 + (100 as usize) {
        printf("  Allocation order and overlap check: PASSED\n", 0 as usize);
    } else {
        printf("  Allocation order and overlap check: FAILED\n", 0 as usize);
    }
    
    // Verify alignment
    if p2 % (16 as usize) == 0 as usize {
        printf("  Alignment check (16-byte): PASSED\n", 0 as usize);
    } else {
        printf("  Alignment check (16-byte): FAILED\n", 0 as usize);
    }
    
    slab::reset(slab);
    let p4 = slab::alloc(slab, 100 as usize, 8 as usize);
    if p4 == p1 {
        printf("  Reset check: PASSED\n", 0 as usize);
    } else {
        printf("  Reset check: FAILED\n", 0 as usize);
    }
    
    printf("Slab test complete.\n", 0 as usize);
}
