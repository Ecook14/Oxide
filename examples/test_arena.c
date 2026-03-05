#include <stdint.h>
#include <stdbool.h>
#include <stdlib.h>
#include <stdio.h>
#include <stdatomic.h>
#include <pthread.h>

typedef struct { uint64_t ptr_data; uint64_t ptr_fn; } Closure;

typedef struct {
    uint64_t next;
    uint64_t capacity;
    uint64_t used;
    uint64_t data;
} ArenaBlock;

typedef struct {
    uint64_t head;
    uint64_t current;
    uint64_t chunk_size;
} Arena;




uint64_t test_basic_alloc();
int main();
uint64_t new_arena(uint64_t r_arg_chunk_size);
uint64_t alloc_block(uint64_t r_arg_capacity);
uint64_t alloc(uint64_t r_arg_self, uint64_t r_arg_size, uint64_t r_arg_align);
uint64_t reset(uint64_t r_arg_self);
uint64_t destroy(uint64_t r_arg_self);


uint64_t test_basic_alloc() {
bb0:
    const char* r_t0 = "--- Basic Alloc ---\n";
    uint64_t r_t1 = 0;
    printf((const char*)r_t0, r_t1);
    uint8_t r_arena_ctx[8];
    uint64_t r_t2 = 128;
    uint64_t r_t3 = (uint64_t)new_arena(r_t2);
    *(uint64_t*)r_arena_ctx = (uint64_t)r_t3;
    uint8_t r_p1[8];
    uint64_t r_t4 = *(uint64_t*)r_arena_ctx;
    uint64_t r_t5 = 32;
    uint64_t r_t6 = 8;
    uint64_t r_t7 = (uint64_t)alloc(r_t4, r_t5, r_t6);
    *(uint64_t*)r_p1 = (uint64_t)r_t7;
    uint8_t r_p2[8];
    uint64_t r_t8 = *(uint64_t*)r_arena_ctx;
    uint64_t r_t9 = 16;
    uint64_t r_t10 = 8;
    uint64_t r_t11 = (uint64_t)alloc(r_t8, r_t9, r_t10);
    *(uint64_t*)r_p2 = (uint64_t)r_t11;
    uint8_t r_p3[8];
    uint64_t r_t12 = *(uint64_t*)r_arena_ctx;
    uint64_t r_t13 = 64;
    uint64_t r_t14 = 16;
    uint64_t r_t15 = (uint64_t)alloc(r_t12, r_t13, r_t14);
    *(uint64_t*)r_p3 = (uint64_t)r_t15;
    uint8_t r_p_big[8];
    uint64_t r_t16 = *(uint64_t*)r_arena_ctx;
    uint64_t r_t17 = 1024;
    uint64_t r_t18 = 8;
    uint64_t r_t19 = (uint64_t)alloc(r_t16, r_t17, r_t18);
    *(uint64_t*)r_p_big = (uint64_t)r_t19;
    uint64_t r_t20 = *(uint64_t*)r_arena_ctx;
    reset(r_t20);
    uint8_t r_p1_new[8];
    uint64_t r_t21 = *(uint64_t*)r_arena_ctx;
    uint64_t r_t22 = 32;
    uint64_t r_t23 = 8;
    uint64_t r_t24 = (uint64_t)alloc(r_t21, r_t22, r_t23);
    *(uint64_t*)r_p1_new = (uint64_t)r_t24;
    uint64_t r_t25 = *(uint64_t*)r_p1;
    uint64_t r_t26 = *(uint64_t*)r_p1_new;
    uint64_t r_t27 = r_t25 == r_t26;
    if (r_t27) goto bb1; else goto bb2;
bb1:
    const char* r_t28 = "  Reset verification: PASSED\n";
    uint64_t r_t29 = 0;
    printf((const char*)r_t28, r_t29);
    goto bb3;
bb2:
    const char* r_t30 = "  Reset verification: FAILED\n";
    uint64_t r_t31 = 0;
    printf((const char*)r_t30, r_t31);
    goto bb3;
bb3:
    uint64_t r_t32 = *(uint64_t*)r_arena_ctx;
    destroy(r_t32);
    const char* r_t33 = "  Arena Drop: COMPLETED\n";
    uint64_t r_t34 = 0;
    printf((const char*)r_t33, r_t34);
    return 0;
}

int main() {
bb4:
    const char* r_t35 = "Starting Arena Tests...\n";
    uint64_t r_t36 = 0;
    printf((const char*)r_t35, r_t36);
    test_basic_alloc();
    const char* r_t37 = "All Tests Completed Successfully.\n";
    uint64_t r_t38 = 0;
    printf((const char*)r_t37, r_t38);
    uint64_t r_t39 = 0;
    return r_t39;
}

uint64_t new_arena(uint64_t r_arg_chunk_size) {
bb5:
    uint8_t r_chunk_size[8];
    *(uint64_t*)r_chunk_size = (uint64_t)r_arg_chunk_size;
    uint8_t r_block[8];
    uint64_t r_t40 = *(uint64_t*)r_chunk_size;
    uint64_t r_t41 = (uint64_t)alloc_block(r_t40);
    *(uint64_t*)r_block = (uint64_t)r_t41;
    uint64_t r_t42 = (uint64_t)calloc(1, sizeof(Arena));
    uint64_t r_t43 = *(uint64_t*)r_block;
    ((Arena*)r_t42)->head = r_t43;
    uint64_t r_t44 = *(uint64_t*)r_block;
    ((Arena*)r_t42)->current = r_t44;
    uint64_t r_t45 = *(uint64_t*)r_chunk_size;
    ((Arena*)r_t42)->chunk_size = r_t45;
    return r_t42;
}

uint64_t alloc_block(uint64_t r_arg_capacity) {
bb6:
    uint8_t r_capacity[8];
    *(uint64_t*)r_capacity = (uint64_t)r_arg_capacity;
    // unsafe block begins
    uint8_t r_block_ptr[8];
    uint64_t r_t46 = 32;
    uint64_t r_t47 = (uint64_t)malloc(r_t46);
    *(uint64_t*)r_block_ptr = (uint64_t)r_t47;
    uint64_t r_t48 = *(uint64_t*)r_block_ptr;
    uint64_t r_t49 = 0;
    uint64_t r_t50 = r_t48 == r_t49;
    if (r_t50) goto bb7; else goto bb8;
bb7:
    printf("\n[Oxide Panic] %s\n", "OOM: malloc failed for ArenaBlock");
    abort();
    uint64_t r_t51 = 0;
    goto bb9;
bb8:
    goto bb9;
bb9:
    uint8_t r_data_ptr[8];
    uint64_t r_t52 = *(uint64_t*)r_capacity;
    uint64_t r_t53 = (uint64_t)malloc(r_t52);
    *(uint64_t*)r_data_ptr = (uint64_t)r_t53;
    uint64_t r_t54 = *(uint64_t*)r_data_ptr;
    uint64_t r_t55 = 0;
    uint64_t r_t56 = r_t54 == r_t55;
    if (r_t56) goto bb10; else goto bb11;
bb10:
    printf("\n[Oxide Panic] %s\n", "OOM: malloc failed for Arena data");
    abort();
    uint64_t r_t57 = 0;
    goto bb12;
bb11:
    goto bb12;
bb12:
    uint8_t r_block_obj[8];
    uint64_t r_t58 = *(uint64_t*)r_block_ptr;
    *(uint64_t*)r_block_obj = (uint64_t)r_t58;
    uint64_t r_t59 = (uint64_t)calloc(1, sizeof(ArenaBlock));
    uint64_t r_t60 = 0;
    ((ArenaBlock*)r_t59)->next = r_t60;
    uint64_t r_t61 = *(uint64_t*)r_capacity;
    ((ArenaBlock*)r_t59)->capacity = r_t61;
    uint64_t r_t62 = 0;
    ((ArenaBlock*)r_t59)->used = r_t62;
    uint64_t r_t63 = *(uint64_t*)r_data_ptr;
    ((ArenaBlock*)r_t59)->data = r_t63;
    uint64_t r_t64 = *(uint64_t*)r_block_obj;
    *(uint64_t*)r_t64 = (uint64_t)r_t59;
    uint64_t r_t65 = *(uint64_t*)r_block_ptr;
    return r_t65;
    // unsafe block ends
    return 0;
}

uint64_t alloc(uint64_t r_arg_self, uint64_t r_arg_size, uint64_t r_arg_align) {
bb13:
    uint8_t r_self[8];
    *(uint64_t*)r_self = (uint64_t)r_arg_self;
    uint8_t r_size[8];
    *(uint64_t*)r_size = (uint64_t)r_arg_size;
    uint8_t r_align[8];
    *(uint64_t*)r_align = (uint64_t)r_arg_align;
    uint8_t r_curr_ptr[8];
    uint64_t r_t66 = *(uint64_t*)r_self;
    uint64_t r_t67 = (uint64_t)((Arena*)r_t66)->current;
    *(uint64_t*)r_curr_ptr = (uint64_t)r_t67;
bb14:
    uint64_t r_t68 = *(uint64_t*)r_curr_ptr;
    uint64_t r_t69 = 0;
    uint64_t r_t70 = r_t68 == r_t69;
    if (r_t70) goto bb16; else goto bb17;
bb16:
    printf("\n[Oxide Panic] %s\n", "Arena corruption: current block is null");
    abort();
    uint64_t r_t71 = 0;
    goto bb18;
bb17:
    goto bb18;
bb18:
    // unsafe block begins
    uint8_t r_curr[8];
    uint64_t r_t72 = *(uint64_t*)r_curr_ptr;
    *(uint64_t*)r_curr = (uint64_t)r_t72;
    uint8_t r_raw_addr[8];
    uint64_t r_t73 = *(uint64_t*)r_curr;
    uint64_t r_t74 = *(uint64_t*)r_t73;
    uint64_t r_t75 = (uint64_t)((ArenaBlock*)r_t74)->data;
    uint64_t r_t76 = *(uint64_t*)r_curr;
    uint64_t r_t77 = *(uint64_t*)r_t76;
    uint64_t r_t78 = (uint64_t)((ArenaBlock*)r_t77)->used;
    uint64_t r_t79 = r_t75 + r_t78;
    *(uint64_t*)r_raw_addr = (uint64_t)r_t79;
    uint8_t r_padding[8];
    uint64_t r_t80 = 0;
    *(uint64_t*)r_padding = (uint64_t)r_t80;
    uint64_t r_t81 = *(uint64_t*)r_raw_addr;
    uint64_t r_t82 = *(uint64_t*)r_align;
    uint64_t r_t83 = (int64_t)r_t81 % (int64_t)r_t82;
    uint64_t r_t84 = 0;
    uint64_t r_t85 = r_t83 != r_t84;
    if (r_t85) goto bb19; else goto bb20;
bb19:
    uint64_t r_t86 = *(uint64_t*)r_align;
    uint64_t r_t87 = *(uint64_t*)r_raw_addr;
    uint64_t r_t88 = *(uint64_t*)r_align;
    uint64_t r_t89 = (int64_t)r_t87 % (int64_t)r_t88;
    uint64_t r_t90 = r_t86 - r_t89;
    *(uint64_t*)r_padding = (uint64_t)r_t90;
    goto bb21;
bb20:
    goto bb21;
bb21:
    uint8_t r_total_needed[8];
    uint64_t r_t91 = *(uint64_t*)r_size;
    uint64_t r_t92 = *(uint64_t*)r_padding;
    uint64_t r_t93 = r_t91 + r_t92;
    *(uint64_t*)r_total_needed = (uint64_t)r_t93;
    uint64_t r_t94 = *(uint64_t*)r_curr;
    uint64_t r_t95 = *(uint64_t*)r_t94;
    uint64_t r_t96 = (uint64_t)((ArenaBlock*)r_t95)->used;
    uint64_t r_t97 = *(uint64_t*)r_total_needed;
    uint64_t r_t98 = r_t96 + r_t97;
    uint64_t r_t99 = *(uint64_t*)r_curr;
    uint64_t r_t100 = *(uint64_t*)r_t99;
    uint64_t r_t101 = (uint64_t)((ArenaBlock*)r_t100)->capacity;
    uint64_t r_t102 = (int64_t)r_t98 <= (int64_t)r_t101;
    if (r_t102) goto bb22; else goto bb23;
bb22:
    uint64_t r_t103 = *(uint64_t*)r_curr;
    uint64_t r_t104 = *(uint64_t*)r_t103;
    uint64_t r_t105 = (uint64_t)((ArenaBlock*)r_t104)->used;
    uint64_t r_t106 = *(uint64_t*)r_total_needed;
    uint64_t r_t107 = r_t105 + r_t106;
    uint64_t r_t108 = *(uint64_t*)r_curr;
    uint64_t r_t109 = *(uint64_t*)r_t108;
    ((ArenaBlock*)r_t109)->used = r_t107;
    uint64_t r_t110 = *(uint64_t*)r_raw_addr;
    uint64_t r_t111 = *(uint64_t*)r_padding;
    uint64_t r_t112 = r_t110 + r_t111;
    return r_t112;
    goto bb24;
bb23:
    goto bb24;
bb24:
    uint64_t r_t113 = *(uint64_t*)r_curr;
    uint64_t r_t114 = *(uint64_t*)r_t113;
    uint64_t r_t115 = (uint64_t)((ArenaBlock*)r_t114)->next;
    uint64_t r_t116 = 0;
    uint64_t r_t117 = r_t115 != r_t116;
    if (r_t117) goto bb25; else goto bb26;
bb25:
    uint64_t r_t118 = *(uint64_t*)r_curr;
    uint64_t r_t119 = *(uint64_t*)r_t118;
    uint64_t r_t120 = (uint64_t)((ArenaBlock*)r_t119)->next;
    *(uint64_t*)r_curr_ptr = (uint64_t)r_t120;
    goto bb27;
bb26:
    uint8_t r_next_cap[8];
    uint64_t r_t121 = *(uint64_t*)r_self;
    uint64_t r_t122 = (uint64_t)((Arena*)r_t121)->chunk_size;
    *(uint64_t*)r_next_cap = (uint64_t)r_t122;
    uint64_t r_t123 = *(uint64_t*)r_size;
    uint64_t r_t124 = *(uint64_t*)r_next_cap;
    uint64_t r_t125 = (int64_t)r_t123 > (int64_t)r_t124;
    if (r_t125) goto bb28; else goto bb29;
bb28:
    uint64_t r_t126 = *(uint64_t*)r_size;
    *(uint64_t*)r_next_cap = (uint64_t)r_t126;
    goto bb30;
bb29:
    goto bb30;
bb30:
    uint8_t r_next_block[8];
    uint64_t r_t127 = *(uint64_t*)r_next_cap;
    uint64_t r_t128 = (uint64_t)alloc_block(r_t127);
    *(uint64_t*)r_next_block = (uint64_t)r_t128;
    uint64_t r_t129 = *(uint64_t*)r_next_block;
    uint64_t r_t130 = *(uint64_t*)r_curr;
    uint64_t r_t131 = *(uint64_t*)r_t130;
    ((ArenaBlock*)r_t131)->next = r_t129;
    uint64_t r_t132 = *(uint64_t*)r_next_block;
    *(uint64_t*)r_curr_ptr = (uint64_t)r_t132;
    goto bb27;
bb27:
    // unsafe block ends
    goto bb14;
bb15:
    return 0;
}

uint64_t reset(uint64_t r_arg_self) {
bb31:
    uint8_t r_self[8];
    *(uint64_t*)r_self = (uint64_t)r_arg_self;
    uint8_t r_curr_ptr[8];
    uint64_t r_t133 = *(uint64_t*)r_self;
    uint64_t r_t134 = (uint64_t)((Arena*)r_t133)->head;
    *(uint64_t*)r_curr_ptr = (uint64_t)r_t134;
bb32:
    uint64_t r_t135 = *(uint64_t*)r_curr_ptr;
    uint64_t r_t136 = 0;
    uint64_t r_t137 = r_t135 != r_t136;
    if (r_t137) goto bb33; else goto bb34;
bb33:
    // unsafe block begins
    uint8_t r_curr[8];
    uint64_t r_t138 = *(uint64_t*)r_curr_ptr;
    *(uint64_t*)r_curr = (uint64_t)r_t138;
    uint64_t r_t139 = 0;
    uint64_t r_t140 = *(uint64_t*)r_curr;
    uint64_t r_t141 = *(uint64_t*)r_t140;
    ((ArenaBlock*)r_t141)->used = r_t139;
    uint64_t r_t142 = *(uint64_t*)r_curr;
    uint64_t r_t143 = *(uint64_t*)r_t142;
    uint64_t r_t144 = (uint64_t)((ArenaBlock*)r_t143)->next;
    *(uint64_t*)r_curr_ptr = (uint64_t)r_t144;
    // unsafe block ends
    goto bb32;
bb34:
    return 0;
}

uint64_t destroy(uint64_t r_arg_self) {
bb35:
    uint8_t r_self[8];
    *(uint64_t*)r_self = (uint64_t)r_arg_self;
    uint8_t r_curr_ptr[8];
    uint64_t r_t145 = *(uint64_t*)r_self;
    uint64_t r_t146 = (uint64_t)((Arena*)r_t145)->head;
    *(uint64_t*)r_curr_ptr = (uint64_t)r_t146;
bb36:
    uint64_t r_t147 = *(uint64_t*)r_curr_ptr;
    uint64_t r_t148 = 0;
    uint64_t r_t149 = r_t147 != r_t148;
    if (r_t149) goto bb37; else goto bb38;
bb37:
    // unsafe block begins
    uint8_t r_curr[8];
    uint64_t r_t150 = *(uint64_t*)r_curr_ptr;
    *(uint64_t*)r_curr = (uint64_t)r_t150;
    uint8_t r_next_ptr[8];
    uint64_t r_t151 = *(uint64_t*)r_curr;
    uint64_t r_t152 = *(uint64_t*)r_t151;
    uint64_t r_t153 = (uint64_t)((ArenaBlock*)r_t152)->next;
    *(uint64_t*)r_next_ptr = (uint64_t)r_t153;
    uint64_t r_t154 = *(uint64_t*)r_curr;
    uint64_t r_t155 = *(uint64_t*)r_t154;
    uint64_t r_t156 = (uint64_t)((ArenaBlock*)r_t155)->data;
    free(r_t156);
    uint64_t r_t157 = *(uint64_t*)r_curr_ptr;
    free(r_t157);
    uint64_t r_t158 = *(uint64_t*)r_next_ptr;
    *(uint64_t*)r_curr_ptr = (uint64_t)r_t158;
    // unsafe block ends
    goto bb36;
bb38:
    return 0;
}

