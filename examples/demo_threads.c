#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>
#include <pthread.h>

typedef struct {
    _Atomic uint64_t current_ptr;
    uint64_t max_size;
    uint64_t region_start;
} SlabAllocator;

typedef struct {
    _Atomic uint64_t head;
    _Atomic uint64_t tail;
    uint64_t buffer;
    uint64_t capacity;
} MpscQueue;

typedef struct {
    uint64_t handle;
} Thread;



typedef struct { uint64_t val; bool ok; } cas_result;

uint64_t dummy_thread(uint64_t r_arg_arg);
uint64_t test_library_resolution();
int main();
uint64_t create_slab(uint64_t r_arg_size);
uint64_t new_queue(uint64_t r_arg_capacity);
uint64_t spawn_raw(uint64_t r_arg_routine, uint64_t r_arg_arg);

uint64_t dummy_thread(uint64_t r_arg_arg) {
bb0:
    uint8_t r_arg[8];
    *(uint64_t*)r_arg = (uint64_t)r_arg_arg;
    uint8_t r_msg[8];
    const char* r_t0 = "Hello from Oxide spawned thread!\n";
    *(uint64_t*)r_msg = (uint64_t)r_t0;
    uint64_t r_t1 = *(uint64_t*)r_msg;
    uint64_t r_t2 = 0;
    int r_t3 = printf((const char*)r_t1, r_t2);
    uint64_t r_t4 = 0;
    return r_t4;
}

uint64_t test_library_resolution() {
bb1:
    uint8_t r_allocator[8];
    uint64_t r_t5 = 4096;
    uint64_t r_t6 = (uint64_t)create_slab(r_t5);
    *(uint64_t*)r_allocator = (uint64_t)r_t6;
    uint8_t r_queue[8];
    uint64_t r_t7 = 100;
    uint64_t r_t8 = (uint64_t)new_queue(r_t7);
    *(uint64_t*)r_queue = (uint64_t)r_t8;
    uint8_t r_handle[8];
    uint64_t r_t9 = (uint64_t)dummy_thread;
    uint64_t r_t10 = 0;
    uint64_t r_t11 = (uint64_t)spawn_raw(r_t9, r_t10);
    *(uint64_t*)r_handle = (uint64_t)r_t11;
    uint64_t r_t12 = *(uint64_t*)r_allocator;
    uint64_t r_t13 = (uint64_t)((SlabAllocator*)r_t12)->max_size;
    return r_t13;
}

int main() {
bb2:
    uint8_t r_result[8];
    uint64_t r_t14 = (uint64_t)test_library_resolution();
    *(uint64_t*)r_result = (uint64_t)r_t14;
    uint8_t r_msg[8];
    const char* r_t15 = "Oxide Phase 2 Standard Library Compilation Success! Slab capacity: %ld\n";
    *(uint64_t*)r_msg = (uint64_t)r_t15;
    uint64_t r_t16 = *(uint64_t*)r_msg;
    uint64_t r_t17 = *(uint64_t*)r_result;
    int r_t18 = printf((const char*)r_t16, r_t17);
    uint64_t r_t19 = 0;
    return r_t19;
}

uint64_t create_slab(uint64_t r_arg_size) {
bb3:
    uint8_t r_size[8];
    *(uint64_t*)r_size = (uint64_t)r_arg_size;
    uint8_t r_actual_ptr[8];
    uint64_t r_t20 = 0;
    *(uint64_t*)r_actual_ptr = (uint64_t)r_t20;
    // unsafe block begins
    uint64_t r_t21 = *(uint64_t*)r_size;
    uint64_t r_t22 = (uint64_t)malloc(r_t21);
    *(uint64_t*)r_actual_ptr = (uint64_t)r_t22;
    uint64_t r_t23 = *(uint64_t*)r_actual_ptr;
    uint64_t r_t24 = 0;
    uint64_t r_t25 = r_t23 == r_t24;
    if (r_t25) goto bb4; else goto bb5;
bb4:
    printf("\n[Oxide Panic] %s\n", "OOM: malloc failed in slab allocator");
    abort();
    uint64_t r_t26 = 0;
    goto bb6;
bb5:
    goto bb6;
bb6:
    // unsafe block ends
    uint64_t r_t27 = (uint64_t)calloc(1, sizeof(SlabAllocator));
    uint64_t r_t28 = *(uint64_t*)r_actual_ptr;
    ((SlabAllocator*)r_t27)->current_ptr = r_t28;
    uint64_t r_t29 = *(uint64_t*)r_size;
    ((SlabAllocator*)r_t27)->max_size = r_t29;
    uint64_t r_t30 = *(uint64_t*)r_actual_ptr;
    ((SlabAllocator*)r_t27)->region_start = r_t30;
    return r_t27;
}

uint64_t new_queue(uint64_t r_arg_capacity) {
bb7:
    uint8_t r_capacity[8];
    *(uint64_t*)r_capacity = (uint64_t)r_arg_capacity;
    uint8_t r_real_buf[8];
    uint64_t r_t31 = 0;
    *(uint64_t*)r_real_buf = (uint64_t)r_t31;
    // unsafe block begins
    uint64_t r_t32 = *(uint64_t*)r_capacity;
    uint64_t r_t33 = 8;
    uint64_t r_t34 = r_t32 * r_t33;
    uint64_t r_t35 = (uint64_t)malloc(r_t34);
    *(uint64_t*)r_real_buf = (uint64_t)r_t35;
    uint64_t r_t36 = *(uint64_t*)r_real_buf;
    uint64_t r_t37 = 0;
    uint64_t r_t38 = r_t36 == r_t37;
    if (r_t38) goto bb8; else goto bb9;
bb8:
    printf("\n[Oxide Panic] %s\n", "OOM: malloc failed in mpsc queue");
    abort();
    uint64_t r_t39 = 0;
    goto bb10;
bb9:
    goto bb10;
bb10:
    // unsafe block ends
    uint64_t r_t40 = (uint64_t)calloc(1, sizeof(MpscQueue));
    uint64_t r_t41 = 0;
    ((MpscQueue*)r_t40)->head = r_t41;
    uint64_t r_t42 = 0;
    ((MpscQueue*)r_t40)->tail = r_t42;
    uint64_t r_t43 = *(uint64_t*)r_real_buf;
    ((MpscQueue*)r_t40)->buffer = r_t43;
    uint64_t r_t44 = *(uint64_t*)r_capacity;
    ((MpscQueue*)r_t40)->capacity = r_t44;
    return r_t40;
}

uint64_t spawn_raw(uint64_t r_arg_routine, uint64_t r_arg_arg) {
bb11:
    uint8_t r_routine[8];
    *(uint64_t*)r_routine = (uint64_t)r_arg_routine;
    uint8_t r_arg[8];
    *(uint64_t*)r_arg = (uint64_t)r_arg_arg;
    uint8_t r_handle_buf[8];
    uint64_t r_t45 = 0;
    *(uint64_t*)r_handle_buf = (uint64_t)r_t45;
    // unsafe block begins
    uint8_t r_res[8];
    uint64_t r_t46 = (uint64_t)r_handle_buf;
    uint64_t r_t47 = 0;
    uint64_t r_t48 = *(uint64_t*)r_routine;
    uint64_t r_t49 = *(uint64_t*)r_arg;
    uint64_t r_t50 = (uint64_t)pthread_create((pthread_t*)r_t46, (const pthread_attr_t*)r_t47, (void*(*)(void*))r_t48, (void*)r_t49);
    *(uint64_t*)r_res = (uint64_t)r_t50;
    uint64_t r_t51 = *(uint64_t*)r_res;
    uint64_t r_t52 = 0;
    uint64_t r_t53 = r_t51 != r_t52;
    if (r_t53) goto bb12; else goto bb13;
bb12:
    printf("\n[Oxide Panic] %s\n", "OS thread creation failed");
    abort();
    uint64_t r_t54 = 0;
    goto bb14;
bb13:
    goto bb14;
bb14:
    // unsafe block ends
    uint64_t r_t55 = (uint64_t)calloc(1, sizeof(Thread));
    uint64_t r_t56 = *(uint64_t*)r_handle_buf;
    ((Thread*)r_t55)->handle = r_t56;
    return r_t55;
}

