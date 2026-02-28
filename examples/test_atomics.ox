fn main() -> u64 {
    let mut x: atomic<u64> = 0;
    
    // Explicit ordering keywords
    x.store(42, relaxed);
    let val: u64 = x.load(acquire);
    
    if val == 42 {
        x.fetch_add(1, seq_cst);
    }
    
    let old: u64 = x.swap(100, acq_rel);
    
    // CAS
    // compare_exchange(expected, new, order)
    // Note: Oxide CAS returns a struct { val: u64, ok: bool } which we can't destructure easily in Phase 1
    // but we can check the return value if we had field access on it.
    let res = x.compare_exchange(100, 200, seq_cst);
    
    return 0;
}
