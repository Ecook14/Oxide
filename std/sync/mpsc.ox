// ============================================================
// std/sync/mpsc.ox — Lock-Free Bounded Queue (MPSC)
// ============================================================
// A high-performance multiple-producer, single-consumer queue
// using sequence-based synchronization for lock-free operations.
// ============================================================

extern "c" fn malloc(size: u64) -> *mut u8;
extern "c" fn abort(msg: *const u8) -> void;

shared struct MpscQueue {
    pub head: atomic<usize>,
    pub tail: atomic<usize>,
    pub buffer: usize,    // Pointer to data array (u64)
    pub sequences: usize, // Pointer to sequence array (atomic<usize>)
    pub capacity: usize,
}

pub fn new_queue(capacity: usize) -> MpscQueue {
    let mut data_ptr = 0 as usize;
    let mut seq_ptr = 0 as usize;
    
    // SAFETY: Allocating buffers for the ring queue.
    unsafe {
        data_ptr = malloc((capacity * 8) as u64) as usize;
        seq_ptr = malloc((capacity * 8) as u64) as usize;
        
        if data_ptr == 0 as usize || seq_ptr == 0 as usize {
            abort("OOM: malloc failed for mpsc queue");
        }
        
        // Initialize sequences: sequences[i] = i
        let mut i = 0 as usize;
        while i < capacity {
            let s_ptr = (seq_ptr + (i * 8)) as *mut atomic<usize>;
            // Direct assignment to atomic pointer to initialize
            *s_ptr = i;
            i = i + 1;
        }
    }
    
    return MpscQueue {
        head: 0,
        tail: 0,
        buffer: data_ptr,
        sequences: seq_ptr,
        capacity: capacity,
    };
}

pub fn enqueue(self: &MpscQueue, val: u64) -> bool {
    loop {
        // SAFETY: Using address-of for atomic operations on shared fields.
        let pos = (&self.tail).load(relaxed);
        let s_ptr = (self.sequences + ((pos % self.capacity) * 8)) as *mut atomic<usize>;
        
        // s_ptr is already a pointer to atomic, so .load() works as intended.
        let seq = s_ptr.load(acquire);
        let diff = (seq as isize) - (pos as isize);
        
        if diff == 0 {
            if (&self.tail).compare_exchange(pos, pos + 1, relaxed) {
                let d_ptr = (self.buffer + ((pos % self.capacity) * 8)) as *mut u64;
                unsafe { *d_ptr = val; }
                s_ptr.store(pos + 1, release);
                return true;
            }
        } else if diff < 0 {
            return false; // Queue is full
        }
    }
}

pub fn dequeue(self: &MpscQueue) -> u64 {
    loop {
        let pos = (&self.head).load(relaxed);
        let s_ptr = (self.sequences + ((pos % self.capacity) * 8)) as *mut atomic<usize>;
        let seq = s_ptr.load(acquire);
        let diff = (seq as isize) - ((pos as isize) + 1);
        
        if diff == 0 {
            let d_ptr = (self.buffer + ((pos % self.capacity) * 8)) as *mut u64;
            let val = unsafe { *d_ptr };
            s_ptr.store(pos + self.capacity, release);
            (&self.head).store(pos + 1, relaxed);
            return val;
        }
    }
}
