# Oxide Standard Library: `std::mem::slab`

The `slab` module provides a **Slab Allocator**, a building block for fast, bump-pointer style region bindings. It bypasses global fragmentation entirely by dedicating fixed-size sequential allocations via lock-free atomic offsets.

## `SlabAllocator`

Because Oxide is defined as a zero-GC language leaning heavily into explicit regions, the Standard Library provides the `SlabAllocator` to service scenarios where objects share exact lifetimes but differ in temporal instantiation constraints.

### Methods

#### `create_slab`
```oxide
pub fn create_slab(size: usize) -> SlabAllocator
```
Delegates a large contiguous chunk to backend `malloc` equal to the specified `size` bytes. The slab keeps an atomic tracking pointer tracking the first available chunk.

#### `alloc`
```oxide
pub fn alloc(self: SlabAllocator, size: usize, align: usize) -> usize
```
Finds the next `size`-byte segment within the pre-warmed Slab that honors the integer `align` constraint (e.g. 8-byte alignment). Uses an `atomic_compare_exchange_strong_explicit` to safely advance the start of the free block, even if multiple threads concurrently attempt to claim sectors. Returns `0` if out of capacity.

#### `reset`
```oxide
pub fn reset(self: SlabAllocator)
```
Invokes a deterministic O(1) bulk de-allocation. Rather than performing `free` on every node, the `current_ptr` atomic is slammed back to 0 utilizing a `memory_order_seq_cst` store, effectively nullifying all extant data inside the Slab safely, reclaiming massive fragmented clusters in single nanosecond cycles.
