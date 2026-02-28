use std::thread;

extern "c" fn printf(format: &u8, val: u64) -> i32;

fn main() -> u64 {
    let x = 12345 as u64;
    
    let handle = thread::spawn(|| {
        printf("Closure thread running! Captured x = %lu\n", x);
    });
    
    // Simple verification: closure object was created and passed to thread::spawn
    printf("Main thread: closure thread spawned.\n", 0 as u64);
    
    return 0;
}
