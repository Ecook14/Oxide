#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>

typedef struct { uint64_t val; bool ok; } cas_result;

uint64_t add(uint64_t r_arg_a, uint64_t r_arg_b) {
bb0:
    uint8_t r_a[8];
    *(uint64_t*)r_a = r_arg_a;
    uint8_t r_b[8];
    *(uint64_t*)r_b = r_arg_b;
    uint64_t r_t0 = *(uint64_t*)r_a;
    uint64_t r_t1 = *(uint64_t*)r_b;
    uint64_t r_t2 = r_t0 + r_t1;
    return r_t2;
}

uint64_t demo_control_flow() {
bb1:
    uint8_t r_x[8];
    uint64_t r_t3 = 10;
    *(uint64_t*)r_x = r_t3;
    uint8_t r_y[8];
    uint64_t r_t4 = 20;
    *(uint64_t*)r_y = r_t4;
    uint64_t r_t5 = *(uint64_t*)r_x;
    uint64_t r_t6 = 10;
    uint64_t r_t7 = r_t5 > r_t6;
    if (r_t7) goto bb2; else goto bb3;
bb2:
    uint64_t r_t8 = 1;
    *(uint64_t*)r_y = r_t8;
    goto bb4;
bb3:
    uint64_t r_t9 = *(uint64_t*)r_x;
    uint64_t r_t10 = 5;
    uint64_t r_t11 = r_t9 == r_t10;
    if (r_t11) goto bb5; else goto bb6;
bb5:
    uint64_t r_t12 = 2;
    *(uint64_t*)r_y = r_t12;
    goto bb7;
bb6:
    uint64_t r_t13 = 3;
    *(uint64_t*)r_y = r_t13;
    goto bb7;
bb7:
    goto bb4;
bb4:
bb8:
    uint64_t r_t14 = *(uint64_t*)r_y;
    uint64_t r_t15 = 0;
    uint64_t r_t16 = r_t14 > r_t15;
    if (r_t16) goto bb9; else goto bb10;
bb9:
    uint64_t r_t17 = 1;
    uint64_t r_t18 = *(uint64_t*)r_y;
    uint64_t r_t19 = r_t18 - r_t17;
    *(uint64_t*)r_y = r_t19;
    goto bb8;
bb10:
bb11:
    uint64_t r_t20 = *(uint64_t*)r_y;
    uint64_t r_t21 = 0;
    uint64_t r_t22 = r_t20 == r_t21;
    if (r_t22) goto bb13; else goto bb14;
bb13:
    goto bb12;
    goto bb15;
bb14:
    goto bb15;
bb15:
    goto bb11;
    goto bb11;
bb12:
    return 0;
}

uint64_t demo_region() {
bb16:
    // Region 'r' begins
    uint8_t r_buf[8];
    uint64_t r_t23 = 1024;
    *(uint64_t*)r_buf = r_t23;
    // Region 'r' freed (O(1))
    return 0;
}

uint64_t demo_unsafe() {
bb17:
    uint8_t r_val[8];
    uint64_t r_t24 = 100;
    *(uint64_t*)r_val = r_t24;
    // unsafe block begins
    uint8_t r_raw[8];
    uint64_t r_t25 = 42;
    *(uint64_t*)r_raw = r_t25;
    // unsafe block ends
    return 0;
}

uint64_t demo_match() {
bb18:
    uint8_t r_res[8];
    uint64_t r_t26 = 10;
    *(uint64_t*)r_res = r_t26;
    return 0;
}

int main() {
bb19:
    uint8_t r_result[8];
    uint64_t r_t27 = 10;
    uint64_t r_t28 = 20;
    uint64_t r_t29 = add(r_t27, r_t28);
    *(uint64_t*)r_result = r_t29;
    uint64_t r_t30 = demo_control_flow();
    uint64_t r_t31 = demo_region();
    uint64_t r_t32 = demo_unsafe();
    const char* r_t33 = "Oxide Demo Completed Successfully! Add result: %ld\n";
    uint64_t r_t34 = *(uint64_t*)r_result;
    printf(r_t33, r_t34);
    return 0;
}

