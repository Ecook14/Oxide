#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>
#include <pthread.h>

typedef struct { uint64_t ptr_data; uint64_t ptr_fn; } Closure;

typedef struct {
    _Atomic uint64_t current_ptr;
    uint64_t max_size;
    uint64_t region_start;
} SlabAllocator;




int main();
uint64_t create_slab(uint64_t r_arg_size);
uint64_t alloc(uint64_t r_arg_self, uint64_t r_arg_size, uint64_t r_arg_align);
uint64_t reset(uint64_t r_arg_self);


int main() {
bb0:
    uint8_t r_slab[8];
    uint64_t r_t0 = 1024;
    uint64_t r_t1 = 1024;
    uint64_t r_t2 = r_t0 * r_t1;
    uint64_t r_t3 = (uint64_t)create_slab(r_t2);
    *(uint64_t*)r_slab = (uint64_t)r_t3;
    uint8_t r_p1[8];
    uint64_t r_t4 = (uint64_t)r_slab;
    uint64_t r_t5 = 100;
    uint64_t r_t6 = 8;
    uint64_t r_t7 = (uint64_t)alloc(r_t4, r_t5, r_t6);
    *(uint64_t*)r_p1 = (uint64_t)r_t7;
    uint8_t r_p2[8];
    uint64_t r_t8 = (uint64_t)r_slab;
    uint64_t r_t9 = 200;
    uint64_t r_t10 = 16;
    uint64_t r_t11 = (uint64_t)alloc(r_t8, r_t9, r_t10);
    *(uint64_t*)r_p2 = (uint64_t)r_t11;
    uint8_t r_p3[8];
    uint64_t r_t12 = (uint64_t)r_slab;
    uint64_t r_t13 = 50;
    uint64_t r_t14 = 8;
    uint64_t r_t15 = (uint64_t)alloc(r_t12, r_t13, r_t14);
    *(uint64_t*)r_p3 = (uint64_t)r_t15;
    const char* r_t16 = "Slab allocations:\n";
    uint64_t r_t17 = 0;
    int r_t18 = printf((const char*)r_t16, r_t17);
    const char* r_t19 = "  p1: %p\n";
    uint64_t r_t20 = *(uint64_t*)r_p1;
    int r_t21 = printf((const char*)r_t19, r_t20);
    const char* r_t22 = "  p2: %p\n";
    uint64_t r_t23 = *(uint64_t*)r_p2;
    int r_t24 = printf((const char*)r_t22, r_t23);
    const char* r_t25 = "  p3: %p\n";
    uint64_t r_t26 = *(uint64_t*)r_p3;
    int r_t27 = printf((const char*)r_t25, r_t26);
    uint64_t r_t28 = *(uint64_t*)r_p2;
    uint64_t r_t29 = *(uint64_t*)r_p1;
    uint64_t r_t30 = 100;
    uint64_t r_t31 = r_t29 + r_t30;
    uint64_t r_t32 = r_t28 >= r_t31;
    if (r_t32) goto bb1; else goto bb2;
bb1:
    const char* r_t33 = "  Allocation order and overlap check: PASSED\n";
    uint64_t r_t34 = 0;
    int r_t35 = printf((const char*)r_t33, r_t34);
    goto bb3;
bb2:
    const char* r_t36 = "  Allocation order and overlap check: FAILED\n";
    uint64_t r_t37 = 0;
    int r_t38 = printf((const char*)r_t36, r_t37);
    goto bb3;
bb3:
    uint64_t r_t39 = *(uint64_t*)r_p2;
    uint64_t r_t40 = 16;
    uint64_t r_t41 = r_t39 % r_t40;
    uint64_t r_t42 = 0;
    uint64_t r_t43 = r_t41 == r_t42;
    if (r_t43) goto bb4; else goto bb5;
bb4:
    const char* r_t44 = "  Alignment check (16-byte): PASSED\n";
    uint64_t r_t45 = 0;
    int r_t46 = printf((const char*)r_t44, r_t45);
    goto bb6;
bb5:
    const char* r_t47 = "  Alignment check (16-byte): FAILED\n";
    uint64_t r_t48 = 0;
    int r_t49 = printf((const char*)r_t47, r_t48);
    goto bb6;
bb6:
    uint64_t r_t50 = (uint64_t)r_slab;
    uint64_t r_t51 = (uint64_t)reset(r_t50);
    uint8_t r_p4[8];
    uint64_t r_t52 = (uint64_t)r_slab;
    uint64_t r_t53 = 100;
    uint64_t r_t54 = 8;
    uint64_t r_t55 = (uint64_t)alloc(r_t52, r_t53, r_t54);
    *(uint64_t*)r_p4 = (uint64_t)r_t55;
    uint64_t r_t56 = *(uint64_t*)r_p4;
    uint64_t r_t57 = *(uint64_t*)r_p1;
    uint64_t r_t58 = r_t56 == r_t57;
    if (r_t58) goto bb7; else goto bb8;
bb7:
    const char* r_t59 = "  Reset check: PASSED\n";
    uint64_t r_t60 = 0;
    int r_t61 = printf((const char*)r_t59, r_t60);
    goto bb9;
bb8:
    const char* r_t62 = "  Reset check: FAILED\n";
    uint64_t r_t63 = 0;
    int r_t64 = printf((const char*)r_t62, r_t63);
    goto bb9;
bb9:
    const char* r_t65 = "Slab test complete.\n";
    uint64_t r_t66 = 0;
    int r_t67 = printf((const char*)r_t65, r_t66);
    return 0;
}

uint64_t create_slab(uint64_t r_arg_size) {
bb10:
    uint8_t r_size[8];
    *(uint64_t*)r_size = (uint64_t)r_arg_size;
    uint8_t r_actual_ptr[8];
    uint64_t r_t68 = 0;
    *(uint64_t*)r_actual_ptr = (uint64_t)r_t68;
    // unsafe block begins
    uint64_t r_t69 = *(uint64_t*)r_size;
    uint64_t r_t70 = (uint64_t)malloc(r_t69);
    *(uint64_t*)r_actual_ptr = (uint64_t)r_t70;
    uint64_t r_t71 = *(uint64_t*)r_actual_ptr;
    uint64_t r_t72 = 0;
    uint64_t r_t73 = r_t71 == r_t72;
    if (r_t73) goto bb11; else goto bb12;
bb11:
    printf("\n[Oxide Panic] %s\n", "OOM: malloc failed in slab allocator");
    abort();
    uint64_t r_t74 = 0;
    goto bb13;
bb12:
    goto bb13;
bb13:
    // unsafe block ends
    uint64_t r_t75 = (uint64_t)calloc(1, sizeof(SlabAllocator));
    uint64_t r_t76 = 0;
    ((SlabAllocator*)r_t75)->current_ptr = r_t76;
    uint64_t r_t77 = *(uint64_t*)r_size;
    ((SlabAllocator*)r_t75)->max_size = r_t77;
    uint64_t r_t78 = *(uint64_t*)r_actual_ptr;
    ((SlabAllocator*)r_t75)->region_start = r_t78;
    return r_t75;
}

uint64_t alloc(uint64_t r_arg_self, uint64_t r_arg_size, uint64_t r_arg_align) {
bb14:
    uint8_t r_self[8];
    *(uint64_t*)r_self = (uint64_t)r_arg_self;
    uint8_t r_size[8];
    *(uint64_t*)r_size = (uint64_t)r_arg_size;
    uint8_t r_align[8];
    *(uint64_t*)r_align = (uint64_t)r_arg_align;
bb15:
    uint8_t r_current[8];
    uint64_t r_t79 = *(uint64_t*)r_self;
    uint64_t r_t80 = (uint64_t)((SlabAllocator*)r_t79)->current_ptr;
    uint64_t r_t82 = *(uint64_t*)r_self;
    uint64_t r_t81 = (uint64_t)&((SlabAllocator*)r_t82)->current_ptr;
    uint64_t r_t83 = atomic_load_explicit((_Atomic uint64_t*)r_t81, memory_order_relaxed);
    *(uint64_t*)r_current = (uint64_t)r_t83;
    uint8_t r_aligned[8];
    uint64_t r_t84 = *(uint64_t*)r_current;
    uint64_t r_t85 = *(uint64_t*)r_align;
    uint64_t r_t86 = r_t84 + r_t85;
    uint64_t r_t87 = 1;
    uint64_t r_t88 = r_t86 - r_t87;
    uint64_t r_t89 = *(uint64_t*)r_align;
    uint64_t r_t90 = 1;
    uint64_t r_t91 = r_t89 - r_t90;
    uint64_t r_t92 = !r_t91;
    uint64_t r_t93 = r_t88 & r_t92;
    *(uint64_t*)r_aligned = (uint64_t)r_t93;
    uint8_t r_next[8];
    uint64_t r_t94 = *(uint64_t*)r_aligned;
    uint64_t r_t95 = *(uint64_t*)r_size;
    uint64_t r_t96 = r_t94 + r_t95;
    *(uint64_t*)r_next = (uint64_t)r_t96;
    uint64_t r_t97 = *(uint64_t*)r_next;
    uint64_t r_t98 = *(uint64_t*)r_self;
    uint64_t r_t99 = (uint64_t)((SlabAllocator*)r_t98)->max_size;
    uint64_t r_t100 = r_t97 > r_t99;
    if (r_t100) goto bb17; else goto bb18;
bb17:
    uint64_t r_t101 = 0;
    return r_t101;
    goto bb19;
bb18:
    goto bb19;
bb19:
    uint64_t r_t102 = *(uint64_t*)r_self;
    uint64_t r_t103 = (uint64_t)((SlabAllocator*)r_t102)->current_ptr;
    uint64_t r_t105 = *(uint64_t*)r_self;
    uint64_t r_t104 = (uint64_t)&((SlabAllocator*)r_t105)->current_ptr;
    uint64_t r_t106 = *(uint64_t*)r_current;
    uint64_t r_t107 = *(uint64_t*)r_next;
    uint64_t _exp_r_t108 = r_t106;
    uint64_t r_t108 = (uint64_t)atomic_compare_exchange_strong_explicit((_Atomic uint64_t*)r_t104, &_exp_r_t108, r_t107, memory_order_seq_cst, memory_order_seq_cst);
    if (r_t108) goto bb20; else goto bb21;
bb20:
    uint64_t r_t109 = *(uint64_t*)r_self;
    uint64_t r_t110 = (uint64_t)((SlabAllocator*)r_t109)->region_start;
    uint64_t r_t111 = *(uint64_t*)r_aligned;
    uint64_t r_t112 = r_t110 + r_t111;
    return r_t112;
    goto bb22;
bb21:
    goto bb22;
bb22:
    goto bb15;
bb16:
    return 0;
}

uint64_t reset(uint64_t r_arg_self) {
bb23:
    uint8_t r_self[8];
    *(uint64_t*)r_self = (uint64_t)r_arg_self;
    uint64_t r_t113 = *(uint64_t*)r_self;
    uint64_t r_t114 = (uint64_t)((SlabAllocator*)r_t113)->current_ptr;
    uint64_t r_t116 = *(uint64_t*)r_self;
    uint64_t r_t115 = (uint64_t)&((SlabAllocator*)r_t116)->current_ptr;
    uint64_t r_t117 = 0;
    atomic_store_explicit((_Atomic uint64_t*)r_t115, r_t117, memory_order_seq_cst);
    uint64_t r_t118 = 0;
    return 0;
}

