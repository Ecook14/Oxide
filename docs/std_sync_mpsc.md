# Oxide Standard Library: `std::sync::mpsc`

The `mpsc` module provides a **M**ultiple **P**roducer, **S**ingle **C**onsumer unbounded or bounded message queue. It acts as the backbone for Oxide's safe message-passing concurrency architecture.

## `MpscQueue`

The `MpscQueue` relies on lock-free, sequence-based synchronization logic leveraging `stdatomic`. 

Because Oxide adheres to a strict "No Global Mutable State" design, queues are explicitly allocated in heap memory using standard standard pointers and decoupled sequences to prevent false-sharing.

### Constructor

```oxide
pub fn new_queue(capacity: usize) -> MpscQueue
```
Allocates a new queue backed by `malloc` with exact capacity `capacity`. A lock-free sequence buffer is independently allocated to mirror the data ring, avoiding synchronization stalls between reading threads.

### Methods

#### `enqueue`
```oxide
pub fn enqueue(self: MpscQueue, val: u64) -> bool
```
Attempts to enqueue a raw 64-bit value to the tail of the queue.
- Safely spinning using atomic sequence verification.
- Returns `true` if enqueued.
- Yields the thread explicitly (`sched_yield()`) back to the execution scheduler upon queue-full boundaries to avoid CPU starvation.

#### `dequeue`
```oxide
pub fn dequeue(self: MpscQueue) -> u64
```
Extracts the next element inserted at the head of the queue.
- Performs `acquire`/`release` memory barrier synchronizations mathematically guaranteeing full visibility of producer commits.
- Uses cooperative `sched_yield` when elements have been claimed by producers but identical sequences have not populated data blocks.
