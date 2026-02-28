fn main() -> u64 {
    let x = 42;
    
    // Minimal closure capturing x
    let clos = |a| {
        let y = a + x;
        return y;
    };
    
    return 0;
}
