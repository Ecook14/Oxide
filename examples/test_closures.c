#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>
#include <pthread.h>

typedef struct { uint64_t ptr_data; uint64_t ptr_fn; } Closure;




int main();
uint64_t _clos_tramp_0(uint64_t r___env, uint64_t r_arg_a);

typedef struct {
    uint64_t x;
} _clos_env_0;


int main() {
bb0:
    uint8_t r_x[8];
    uint64_t r_t0 = 42;
    *(uint64_t*)r_x = (uint64_t)r_t0;
    uint8_t r_clos[8];
    uint64_t r_t1 = (uint64_t)calloc(1, sizeof(_clos_env_0));
    uint64_t r_t2 = *(uint64_t*)r_x;
    ((_clos_env_0*)r_t1)->x = r_t2;
    uint64_t r_t3 = (uint64_t)_clos_tramp_0;
    uint64_t r_t4 = (uint64_t)calloc(1, sizeof(Closure));
    ((Closure*)r_t4)->ptr_data = r_t1;
    ((Closure*)r_t4)->ptr_fn = r_t3;
    *(uint64_t*)r_clos = (uint64_t)r_t4;
    uint64_t r_t5 = 0;
    return r_t5;
}

uint64_t _clos_tramp_0(uint64_t r___env, uint64_t r_arg_a) {
bb1:
    uint8_t r_x[8];
    uint64_t r_t6 = (uint64_t)((_clos_env_0*)r___env)->x;
    *(uint64_t*)r_x = (uint64_t)r_t6;
    uint8_t r_a[8];
    *(uint64_t*)r_a = (uint64_t)r_arg_a;
    uint8_t r_y[8];
    uint64_t r_t7 = *(uint64_t*)r_a;
    uint64_t r_t8 = *(uint64_t*)r_x;
    uint64_t r_t9 = r_t7 + r_t8;
    *(uint64_t*)r_y = (uint64_t)r_t9;
    uint64_t r_t10 = *(uint64_t*)r_y;
    return r_t10;
}

