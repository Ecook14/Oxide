// ============================================================
// std/thread.ox — C-ABI POSIX Thread Wrapper
// ============================================================
// Zero-cost abstraction binding local closure executions to
// native system threads (e.g. pthread_create on Unix).
// ============================================================

extern "c" fn pthread_create(thread: *mut u8, attr: *u8, start_routine: *u8, arg: *u8) -> i32;
extern "c" fn pthread_join(thread: *u8, retval: *mut u8) -> i32;

pub struct Thread {
    pub handle: u64,
}

// @unstable - Awaiting closure memory escape analysis.
// Oxide closures do not yet lower dynamically allocated heap trampolines or capture structs.
// This raw spawn function binds directly to POSIX strictly to validate FFI correctness.
pub fn spawn_raw(routine: *u8, arg: *u8) -> Thread {
    let mut handle_buf = 0 as u64; 
    
    // SAFETY: We pass a mutable pointer to handle_buf which is exactly 8 bytes (u64).
    // pthread_create will populate it with the OS thread handle.
    // The routine parameter acts as the thread's start_routine. 
    // Ownership of `arg` implicitly transfers over the OS boundary, which must be tracked.
    unsafe {
        // Assume *mut u8 cast is internally checked at compiler boundary
        let res = pthread_create((&mut handle_buf) as *mut u8, 0 as *u8, routine, arg);
        if res != 0 as i32 {
            // Error handling doctrine: map OS returns or abort deterministically.
            abort("OS thread creation failed");
        }
    }
    
    return Thread { handle: handle_buf }; 
}

// Higher-level spawn that accepts a closure.
pub fn spawn(f: |()| -> ()) -> Thread {
    // SAFETY: We decompose the closure into its C-ABI components (env ptr and tramp ptr).
    // The magic fields 'ptr_fn' and 'ptr_data' are provided by the compiler for this purpose.
    unsafe {
        return spawn_raw(f.ptr_fn as *u8, f.ptr_data as *u8);
    }
}
