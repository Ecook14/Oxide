use std::sync::mpsc;
use std::thread;

extern "c" fn printf(fmt: *const u8, arg1: u64) -> i32;

fn producer(queue: &mpsc::MpscQueue, start: u64, count: u64) {
    let mut i = 0 as u64;
    while i < count {
        let val = start + i;
        if queue.enqueue(val) {
            i = i + 1;
        }
        // If full, busy wait (simplified for test)
    }
}

fn main() {
    let queue = mpsc::new_queue(1024 as usize);
    let q_ref = &queue;
    
    printf("Starting MPSC test...\n");
    
    // Spawn two producers
    thread::spawn(|| {
        producer(q_ref, 1000 as u64, 100 as u64);
    });
    
    thread::spawn(|| {
        producer(q_ref, 2000 as u64, 100 as u64);
    });
    
    // Consumer in main thread
    let mut total_received = 0 as u64;
    let mut sum = 0 as u64;
    
    while total_received < 200 as u64 {
        let val = queue.dequeue();
        sum = sum + val;
        total_received = total_received + 1;
        
        if total_received % (50 as u64) == 0 as u64 {
            printf("  Received %llu messages...\n", total_received);
        }
    }
    
    printf("MPSC Test Complete.\n");
    printf("  Total received: %llu\n", total_received);
    printf("  Sum of values: %llu\n", sum);
    
    // Expected sum: sum(1000..1099) + sum(2000..2099)
    // 1000*100 + 4950 + 2000*100 + 4950 = 100000 + 4950 + 200000 + 4950 = 309900
    if sum == 309900 as u64 {
        printf("  Verification: SUCCESS\n");
    } else {
        printf("  Verification: FAILED (Expected 309900)\n");
    }
}
 village
