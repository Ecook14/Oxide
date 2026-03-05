// ============================================================
// std/mem/arena.ox — Multi-Region Hierarchical Arena
// ============================================================
// A fast, deterministic bump-allocator that supports dynamic
// growth via a linked list of allocation blocks.
// ============================================================

extern "c" fn malloc(size: u64) -> *mut u8;
extern "c" fn free(ptr: *mut u8) -> void;
extern "c" fn abort(msg: *u8) -> void;

struct ArenaBlock {
    pub next: usize, // Pointer to next ArenaBlock
    pub capacity: usize,
    pub used: usize,
    pub data: usize, // Pointer to raw byte array
}

pub struct Arena {
    pub head: usize, // Pointer to first ArenaBlock
    pub current: usize, // Pointer to active ArenaBlock
    pub chunk_size: usize,
}

pub fn new_arena(chunk_size: usize) -> Arena {
    let block = alloc_block(chunk_size);
    return Arena {
        head: block,
        current: block,
        chunk_size: chunk_size,
    };
}

fn alloc_block(capacity: usize) -> usize {
    // SAFETY: We allocate the struct metadata and its backing array.
    unsafe {
        let block_ptr = malloc(32 as u64) as usize; // sizeof(ArenaBlock)
        if block_ptr == 0 as usize {
            abort("OOM: malloc failed for ArenaBlock");
        }
        
        let data_ptr = malloc(capacity as u64) as usize;
        if data_ptr == 0 as usize {
            abort("OOM: malloc failed for Arena data");
        }
        
        let mut block_obj = block_ptr as *mut ArenaBlock;
        *block_obj = ArenaBlock {
            next: 0,
            capacity: capacity,
            used: 0,
            data: data_ptr,
        };
        
        return block_ptr;
    }
}

pub fn alloc(self: Arena, size: usize, align: usize) -> usize {
    let mut curr_ptr = self.current;
    
    loop {
        if curr_ptr == 0 as usize {
            // Should never happen unless arena is corrupted
            abort("Arena corruption: current block is null");
        }
        
        unsafe {
            let curr = curr_ptr as *mut ArenaBlock;
            
            // Calculate alignment padding
            let raw_addr = (*curr).data + (*curr).used;
            let mut padding = 0 as usize;
            
            if (raw_addr % align) != 0 as usize {
                padding = align - (raw_addr % align);
            }
            
            let total_needed = size + padding;
            
            // Does this block have enough space?
            if ((*curr).used + total_needed) <= (*curr).capacity {
                (*curr).used = (*curr).used + total_needed;
                return raw_addr + padding;
            }
            
            // Not enough space. Check if there's a next block.
            if (*curr).next != 0 as usize {
                curr_ptr = (*curr).next;
            } else {
                // We need to allocate a new block.
                let mut next_cap = self.chunk_size;
                if size > next_cap {
                    next_cap = size; // Handle oversized allocations natively
                }
                
                let next_block = alloc_block(next_cap);
                (*curr).next = next_block;
                
                // Advance current and try again
                curr_ptr = next_block;
                // (Instead of updating self.current implicitly which requires mutable reference passing, 
                // we leave self.current alone in Oxide v0.1 mapping. Future allocations will just
                // re-traverse the short exhausted list quickly. For a real prod system, self.current 
                // would be a pointer to atomic or mutated directly.)
            }
        }
    }
}

pub fn reset(self: Arena) {
    let mut curr_ptr = self.head;
    while curr_ptr != 0 as usize {
        unsafe {
            let curr = curr_ptr as *mut ArenaBlock;
            (*curr).used = 0;
            curr_ptr = (*curr).next;
        }
    }
}

pub fn destroy(self: Arena) {
    let mut curr_ptr = self.head;
    while curr_ptr != 0 as usize {
        unsafe {
            let curr = curr_ptr as *mut ArenaBlock;
            let next_ptr = (*curr).next;
            
            free((*curr).data as *mut u8);
            free(curr_ptr as *mut u8);
            
            curr_ptr = next_ptr;
        }
    }
}
