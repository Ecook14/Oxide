// ============================================================
// std/mem/slab.ox — Regional Slab Allocator
// ============================================================
// A high-performance lock-free continuous block allocator
// designed to run inside explicit regional memory boundaries
// to guarantee O(1) bulk deallocation.
// ============================================================

extern "c" fn malloc(size: u64) -> *mut u8;
extern "c" fn abort(msg: *const u8) -> void;

pub shared struct SlabAllocator {
    pub current_ptr: atomic<usize>,
    pub max_size: u64,
    pub region_start: usize,
}

pub fn create_slab(size: usize) -> SlabAllocator {
    let mut actual_ptr = 0 as usize;
    
    // SAFETY: Requesting raw bulk memory from the OS allocator.
    unsafe {
        actual_ptr = malloc(size as u64) as usize;
        if actual_ptr == 0 as usize {
            abort("OOM: malloc failed in slab allocator");
        }
    }
    
    return SlabAllocator {
        current_ptr: 0, 
        max_size: size,
        region_start: actual_ptr,
    };
}

pub fn alloc(self: &SlabAllocator, size: usize, align: usize) -> usize {
    loop {
        // SAFETY: Using address-of operator to ensure atomic operation on the pointer.
        let current = (&self.current_ptr).load(relaxed);
        let aligned = (current + align - 1) & !(align - 1);
        let next = aligned + size;
        
        if next > (self.max_size as usize) {
            return 0 as usize; // OOM in this slab
        }
        
        if (&self.current_ptr).compare_exchange(current, next, seq_cst) {
            return self.region_start + aligned;
        }
    }
}

pub fn reset(self: &SlabAllocator) {
    (&self.current_ptr).store(0, seq_cst);
}
