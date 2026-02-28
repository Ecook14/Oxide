#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>
#include <pthread.h>

typedef struct { uint64_t ptr_data; uint64_t ptr_fn; } Closure;




int main();


int main() {
bb0:
    uint8_t r_x[8];
    uint64_t r_t0 = 0;
    *(uint64_t*)r_x = (uint64_t)r_t0;
    uint64_t r_t1 = *(uint64_t*)r_x;
    uint64_t r_t2 = 42;
    atomic_store_explicit((_Atomic uint64_t*)r_x, r_t2, memory_order_relaxed);
    uint64_t r_t3 = 0;
    uint8_t r_val[8];
    uint64_t r_t4 = *(uint64_t*)r_x;
    uint64_t r_t5 = atomic_load_explicit((_Atomic uint64_t*)r_x, memory_order_acquire);
    *(uint64_t*)r_val = (uint64_t)r_t5;
    uint64_t r_t6 = *(uint64_t*)r_val;
    uint64_t r_t7 = 42;
    uint64_t r_t8 = r_t6 == r_t7;
    if (r_t8) goto bb1; else goto bb2;
bb1:
    uint64_t r_t9 = *(uint64_t*)r_x;
    uint64_t r_t10 = 1;
    uint64_t r_t11 = atomic_fetch_add_explicit((_Atomic uint64_t*)r_x, r_t10, memory_order_seq_cst);
    goto bb3;
bb2:
    goto bb3;
bb3:
    uint8_t r_old[8];
    uint64_t r_t12 = *(uint64_t*)r_x;
    uint64_t r_t13 = 100;
    uint64_t r_t14 = atomic_exchange_explicit((_Atomic uint64_t*)r_x, r_t13, memory_order_acq_rel);
    *(uint64_t*)r_old = (uint64_t)r_t14;
    uint8_t r_res[8];
    uint64_t r_t15 = *(uint64_t*)r_x;
    uint64_t r_t16 = 100;
    uint64_t r_t17 = 200;
    uint64_t _exp_r_t18 = r_t16;
    uint64_t r_t18 = (uint64_t)atomic_compare_exchange_strong_explicit((_Atomic uint64_t*)r_x, &_exp_r_t18, r_t17, memory_order_seq_cst, memory_order_seq_cst);
    *(uint64_t*)r_res = (uint64_t)r_t18;
    uint64_t r_t19 = 0;
    return r_t19;
}

