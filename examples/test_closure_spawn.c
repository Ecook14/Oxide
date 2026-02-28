#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>
#include <pthread.h>

typedef struct { uint64_t ptr_data; uint64_t ptr_fn; } Closure;

typedef struct {
    uint64_t handle;
} Thread;




int main();
uint64_t spawn_raw(uint64_t r_arg_routine, uint64_t r_arg_arg);
uint64_t spawn(uint64_t r_arg_f);
uint64_t _clos_tramp_0(uint64_t r___env);

typedef struct {
    uint64_t x;
} _clos_env_0;


int main() {
bb0:
    uint8_t r_x[8];
    uint64_t r_t0 = 12345;
    *(uint64_t*)r_x = (uint64_t)r_t0;
    uint8_t r_handle[8];
    uint64_t r_t1 = (uint64_t)calloc(1, sizeof(_clos_env_0));
    uint64_t r_t2 = *(uint64_t*)r_x;
    ((_clos_env_0*)r_t1)->x = r_t2;
    uint64_t r_t3 = (uint64_t)_clos_tramp_0;
    uint64_t r_t4 = (uint64_t)calloc(1, sizeof(Closure));
    ((Closure*)r_t4)->ptr_data = r_t1;
    ((Closure*)r_t4)->ptr_fn = r_t3;
    uint64_t r_t5 = (uint64_t)spawn(r_t4);
    *(uint64_t*)r_handle = (uint64_t)r_t5;
    const char* r_t6 = "Main thread: closure thread spawned.\n";
    uint64_t r_t7 = 0;
    int r_t8 = printf((const char*)r_t6, r_t7);
    uint64_t r_t9 = 0;
    return r_t9;
}

uint64_t spawn_raw(uint64_t r_arg_routine, uint64_t r_arg_arg) {
bb1:
    uint8_t r_routine[8];
    *(uint64_t*)r_routine = (uint64_t)r_arg_routine;
    uint8_t r_arg[8];
    *(uint64_t*)r_arg = (uint64_t)r_arg_arg;
    uint8_t r_handle_buf[8];
    uint64_t r_t10 = 0;
    *(uint64_t*)r_handle_buf = (uint64_t)r_t10;
    // unsafe block begins
    uint8_t r_res[8];
    uint64_t r_t11 = (uint64_t)r_handle_buf;
    uint64_t r_t12 = 0;
    uint64_t r_t13 = *(uint64_t*)r_routine;
    uint64_t r_t14 = *(uint64_t*)r_arg;
    uint64_t r_t15 = (uint64_t)pthread_create((pthread_t*)r_t11, (const pthread_attr_t*)r_t12, (void*(*)(void*))r_t13, (void*)r_t14);
    *(uint64_t*)r_res = (uint64_t)r_t15;
    uint64_t r_t16 = *(uint64_t*)r_res;
    uint64_t r_t17 = 0;
    uint64_t r_t18 = r_t16 != r_t17;
    if (r_t18) goto bb2; else goto bb3;
bb2:
    printf("\n[Oxide Panic] %s\n", "OS thread creation failed");
    abort();
    uint64_t r_t19 = 0;
    goto bb4;
bb3:
    goto bb4;
bb4:
    // unsafe block ends
    uint64_t r_t20 = (uint64_t)calloc(1, sizeof(Thread));
    uint64_t r_t21 = *(uint64_t*)r_handle_buf;
    ((Thread*)r_t20)->handle = r_t21;
    return r_t20;
}

uint64_t spawn(uint64_t r_arg_f) {
bb5:
    uint8_t r_f[8];
    *(uint64_t*)r_f = (uint64_t)r_arg_f;
    // unsafe block begins
    uint64_t r_t22 = *(uint64_t*)r_f;
    uint64_t r_t23 = (uint64_t)((Closure*)r_t22)->ptr_fn;
    uint64_t r_t24 = *(uint64_t*)r_f;
    uint64_t r_t25 = (uint64_t)((Closure*)r_t24)->ptr_data;
    uint64_t r_t26 = (uint64_t)spawn_raw(r_t23, r_t25);
    return r_t26;
    // unsafe block ends
    return 0;
}

uint64_t _clos_tramp_0(uint64_t r___env) {
bb6:
    uint8_t r_x[8];
    uint64_t r_t27 = (uint64_t)((_clos_env_0*)r___env)->x;
    *(uint64_t*)r_x = (uint64_t)r_t27;
    const char* r_t28 = "Closure thread running! Captured x = %lu\n";
    uint64_t r_t29 = *(uint64_t*)r_x;
    int r_t30 = printf((const char*)r_t28, r_t29);
    return 0;
}

