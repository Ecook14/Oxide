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

typedef struct {
    _Atomic uint64_t head;
    _Atomic uint64_t tail;
    uint64_t buffer;
    uint64_t sequences;
    uint64_t capacity;
} MpscQueue;




uint64_t producer(uint64_t r_arg_queue, uint64_t r_arg_start, uint64_t r_arg_count);
int main();
uint64_t spawn_raw(uint64_t r_arg_routine, uint64_t r_arg_arg);
uint64_t spawn(uint64_t r_arg_f);
uint64_t new_queue(uint64_t r_arg_capacity);
uint64_t enqueue(uint64_t r_arg_self, uint64_t r_arg_val);
uint64_t dequeue(uint64_t r_arg_self);
uint64_t _clos_tramp_0(uint64_t r___env);
uint64_t _clos_tramp_1(uint64_t r___env);

typedef struct {
    uint64_t q_ref;
} _clos_env_1;

typedef struct {
    uint64_t q_ref;
} _clos_env_0;


uint64_t producer(uint64_t r_arg_queue, uint64_t r_arg_start, uint64_t r_arg_count) {
bb0:
    uint8_t r_queue[8];
    *(uint64_t*)r_queue = (uint64_t)r_arg_queue;
    uint8_t r_start[8];
    *(uint64_t*)r_start = (uint64_t)r_arg_start;
    uint8_t r_count[8];
    *(uint64_t*)r_count = (uint64_t)r_arg_count;
    uint8_t r_i[8];
    uint64_t r_t0 = 0;
    *(uint64_t*)r_i = (uint64_t)r_t0;
bb1:
    uint64_t r_t1 = *(uint64_t*)r_i;
    uint64_t r_t2 = *(uint64_t*)r_count;
    uint64_t r_t3 = (int64_t)r_t1 < (int64_t)r_t2;
    if (r_t3) goto bb2; else goto bb3;
bb2:
    uint8_t r_val[8];
    uint64_t r_t4 = *(uint64_t*)r_start;
    uint64_t r_t5 = *(uint64_t*)r_i;
    uint64_t r_t6 = r_t4 + r_t5;
    *(uint64_t*)r_val = (uint64_t)r_t6;
    uint64_t r_t7 = *(uint64_t*)r_queue;
    uint64_t r_t8 = *(uint64_t*)r_val;
    uint64_t r_t9 = (uint64_t)enqueue(r_t7, r_t8);
    if (r_t9) goto bb4; else goto bb5;
bb4:
    uint64_t r_t10 = *(uint64_t*)r_i;
    uint64_t r_t11 = 1;
    uint64_t r_t12 = r_t10 + r_t11;
    *(uint64_t*)r_i = (uint64_t)r_t12;
    goto bb6;
bb5:
    uint64_t r_t13 = (uint64_t)sched_yield();
    goto bb6;
bb6:
    goto bb1;
bb3:
    return 0;
}

int main() {
bb7:
    uint8_t r_queue[8];
    uint64_t r_t14 = 1024;
    uint64_t r_t15 = (uint64_t)new_queue(r_t14);
    *(uint64_t*)r_queue = (uint64_t)r_t15;
    uint8_t r_q_ref[8];
    uint64_t r_t16 = *(uint64_t*)r_queue;
    *(uint64_t*)r_q_ref = (uint64_t)r_t16;
    const char* r_t17 = "Starting MPSC test...\n";
    uint64_t r_t18 = 0;
    int r_t19 = printf((const char*)r_t17, r_t18);
    uint64_t r_t20 = (uint64_t)calloc(1, sizeof(_clos_env_0));
    uint64_t r_t21 = *(uint64_t*)r_q_ref;
    ((_clos_env_0*)r_t20)->q_ref = r_t21;
    uint64_t r_t22 = (uint64_t)_clos_tramp_0;
    uint64_t r_t23 = (uint64_t)calloc(1, sizeof(Closure));
    ((Closure*)r_t23)->ptr_data = r_t20;
    ((Closure*)r_t23)->ptr_fn = r_t22;
    uint64_t r_t24 = (uint64_t)spawn(r_t23);
    uint64_t r_t25 = (uint64_t)calloc(1, sizeof(_clos_env_1));
    uint64_t r_t26 = *(uint64_t*)r_q_ref;
    ((_clos_env_1*)r_t25)->q_ref = r_t26;
    uint64_t r_t27 = (uint64_t)_clos_tramp_1;
    uint64_t r_t28 = (uint64_t)calloc(1, sizeof(Closure));
    ((Closure*)r_t28)->ptr_data = r_t25;
    ((Closure*)r_t28)->ptr_fn = r_t27;
    uint64_t r_t29 = (uint64_t)spawn(r_t28);
    uint8_t r_total_received[8];
    uint64_t r_t30 = 0;
    *(uint64_t*)r_total_received = (uint64_t)r_t30;
    uint8_t r_sum[8];
    uint64_t r_t31 = 0;
    *(uint64_t*)r_sum = (uint64_t)r_t31;
bb8:
    uint64_t r_t32 = *(uint64_t*)r_total_received;
    uint64_t r_t33 = 200;
    uint64_t r_t34 = (int64_t)r_t32 < (int64_t)r_t33;
    if (r_t34) goto bb9; else goto bb10;
bb9:
    uint8_t r_val[8];
    uint64_t r_t35 = *(uint64_t*)r_queue;
    uint64_t r_t36 = (uint64_t)dequeue(r_t35);
    *(uint64_t*)r_val = (uint64_t)r_t36;
    uint64_t r_t37 = *(uint64_t*)r_sum;
    uint64_t r_t38 = *(uint64_t*)r_val;
    uint64_t r_t39 = r_t37 + r_t38;
    *(uint64_t*)r_sum = (uint64_t)r_t39;
    uint64_t r_t40 = *(uint64_t*)r_total_received;
    uint64_t r_t41 = 1;
    uint64_t r_t42 = r_t40 + r_t41;
    *(uint64_t*)r_total_received = (uint64_t)r_t42;
    uint64_t r_t43 = *(uint64_t*)r_total_received;
    uint64_t r_t44 = 50;
    uint64_t r_t45 = (int64_t)r_t43 % (int64_t)r_t44;
    uint64_t r_t46 = 0;
    uint64_t r_t47 = r_t45 == r_t46;
    if (r_t47) goto bb11; else goto bb12;
bb11:
    const char* r_t48 = "  Received %llu messages...\n";
    uint64_t r_t49 = *(uint64_t*)r_total_received;
    int r_t50 = printf((const char*)r_t48, r_t49);
    goto bb13;
bb12:
    goto bb13;
bb13:
    goto bb8;
bb10:
    const char* r_t51 = "MPSC Test Complete.\n";
    uint64_t r_t52 = 0;
    int r_t53 = printf((const char*)r_t51, r_t52);
    const char* r_t54 = "  Total received: %llu\n";
    uint64_t r_t55 = *(uint64_t*)r_total_received;
    int r_t56 = printf((const char*)r_t54, r_t55);
    const char* r_t57 = "  Sum of values: %llu\n";
    uint64_t r_t58 = *(uint64_t*)r_sum;
    int r_t59 = printf((const char*)r_t57, r_t58);
    uint64_t r_t60 = *(uint64_t*)r_sum;
    uint64_t r_t61 = 309900;
    uint64_t r_t62 = r_t60 == r_t61;
    if (r_t62) goto bb14; else goto bb15;
bb14:
    const char* r_t63 = "  Verification: SUCCESS\n";
    uint64_t r_t64 = 0;
    int r_t65 = printf((const char*)r_t63, r_t64);
    goto bb16;
bb15:
    const char* r_t66 = "  Verification: FAILED (Expected 309900)\n";
    uint64_t r_t67 = 0;
    int r_t68 = printf((const char*)r_t66, r_t67);
    goto bb16;
bb16:
    return 0;
}

uint64_t spawn_raw(uint64_t r_arg_routine, uint64_t r_arg_arg) {
bb17:
    uint8_t r_routine[8];
    *(uint64_t*)r_routine = (uint64_t)r_arg_routine;
    uint8_t r_arg[8];
    *(uint64_t*)r_arg = (uint64_t)r_arg_arg;
    uint8_t r_handle_buf[8];
    uint64_t r_t69 = 0;
    *(uint64_t*)r_handle_buf = (uint64_t)r_t69;
    // unsafe block begins
    uint8_t r_res[8];
    uint64_t r_t70 = (uint64_t)r_handle_buf;
    uint64_t r_t71 = 0;
    uint64_t r_t72 = *(uint64_t*)r_routine;
    uint64_t r_t73 = *(uint64_t*)r_arg;
    uint64_t r_t74 = (uint64_t)pthread_create((pthread_t*)r_t70, (const pthread_attr_t*)r_t71, (void*(*)(void*))r_t72, (void*)r_t73);
    *(uint64_t*)r_res = (uint64_t)r_t74;
    uint64_t r_t75 = *(uint64_t*)r_res;
    uint64_t r_t76 = 0;
    uint64_t r_t77 = r_t75 != r_t76;
    if (r_t77) goto bb18; else goto bb19;
bb18:
    printf("\n[Oxide Panic] %s\n", "OS thread creation failed");
    abort();
    uint64_t r_t78 = 0;
    goto bb20;
bb19:
    goto bb20;
bb20:
    // unsafe block ends
    uint64_t r_t79 = (uint64_t)calloc(1, sizeof(Thread));
    uint64_t r_t80 = *(uint64_t*)r_handle_buf;
    ((Thread*)r_t79)->handle = r_t80;
    return r_t79;
}

uint64_t spawn(uint64_t r_arg_f) {
bb21:
    uint8_t r_f[8];
    *(uint64_t*)r_f = (uint64_t)r_arg_f;
    // unsafe block begins
    uint64_t r_t81 = *(uint64_t*)r_f;
    uint64_t r_t82 = (uint64_t)((Closure*)r_t81)->ptr_fn;
    uint64_t r_t83 = *(uint64_t*)r_f;
    uint64_t r_t84 = (uint64_t)((Closure*)r_t83)->ptr_data;
    uint64_t r_t85 = (uint64_t)spawn_raw(r_t82, r_t84);
    return r_t85;
    // unsafe block ends
    return 0;
}

uint64_t new_queue(uint64_t r_arg_capacity) {
bb22:
    uint8_t r_capacity[8];
    *(uint64_t*)r_capacity = (uint64_t)r_arg_capacity;
    uint8_t r_data_ptr[8];
    uint64_t r_t86 = 0;
    *(uint64_t*)r_data_ptr = (uint64_t)r_t86;
    uint8_t r_seq_ptr[8];
    uint64_t r_t87 = 0;
    *(uint64_t*)r_seq_ptr = (uint64_t)r_t87;
    // unsafe block begins
    uint64_t r_t88 = *(uint64_t*)r_capacity;
    uint64_t r_t89 = 8;
    uint64_t r_t90 = r_t88 * r_t89;
    uint64_t r_t91 = (uint64_t)malloc(r_t90);
    *(uint64_t*)r_data_ptr = (uint64_t)r_t91;
    uint64_t r_t92 = *(uint64_t*)r_capacity;
    uint64_t r_t93 = 8;
    uint64_t r_t94 = r_t92 * r_t93;
    uint64_t r_t95 = (uint64_t)malloc(r_t94);
    *(uint64_t*)r_seq_ptr = (uint64_t)r_t95;
    uint64_t r_t96 = *(uint64_t*)r_data_ptr;
    uint64_t r_t97 = 0;
    uint64_t r_t98 = r_t96 == r_t97;
    uint64_t r_t99 = *(uint64_t*)r_seq_ptr;
    uint64_t r_t100 = 0;
    uint64_t r_t101 = r_t99 == r_t100;
    uint64_t r_t102 = r_t98 | r_t101;
    if (r_t102) goto bb23; else goto bb24;
bb23:
    printf("\n[Oxide Panic] %s\n", "OOM: malloc failed for mpsc queue");
    abort();
    uint64_t r_t103 = 0;
    goto bb25;
bb24:
    goto bb25;
bb25:
    uint8_t r_i[8];
    uint64_t r_t104 = 0;
    *(uint64_t*)r_i = (uint64_t)r_t104;
bb26:
    uint64_t r_t105 = *(uint64_t*)r_i;
    uint64_t r_t106 = *(uint64_t*)r_capacity;
    uint64_t r_t107 = (int64_t)r_t105 < (int64_t)r_t106;
    if (r_t107) goto bb27; else goto bb28;
bb27:
    uint8_t r_s_ptr[8];
    uint64_t r_t108 = *(uint64_t*)r_seq_ptr;
    uint64_t r_t109 = *(uint64_t*)r_i;
    uint64_t r_t110 = 8;
    uint64_t r_t111 = r_t109 * r_t110;
    uint64_t r_t112 = r_t108 + r_t111;
    *(uint64_t*)r_s_ptr = (uint64_t)r_t112;
    uint64_t r_t113 = *(uint64_t*)r_i;
    uint64_t r_t114 = *(uint64_t*)r_s_ptr;
    uint64_t r_t115 = *(uint64_t*)r_s_ptr;
    *(uint64_t*)r_t115 = (uint64_t)r_t113;
    uint64_t r_t116 = *(uint64_t*)r_i;
    uint64_t r_t117 = 1;
    uint64_t r_t118 = r_t116 + r_t117;
    *(uint64_t*)r_i = (uint64_t)r_t118;
    goto bb26;
bb28:
    // unsafe block ends
    uint64_t r_t119 = (uint64_t)calloc(1, sizeof(MpscQueue));
    uint64_t r_t120 = 0;
    ((MpscQueue*)r_t119)->head = r_t120;
    uint64_t r_t121 = 0;
    ((MpscQueue*)r_t119)->tail = r_t121;
    uint64_t r_t122 = *(uint64_t*)r_data_ptr;
    ((MpscQueue*)r_t119)->buffer = r_t122;
    uint64_t r_t123 = *(uint64_t*)r_seq_ptr;
    ((MpscQueue*)r_t119)->sequences = r_t123;
    uint64_t r_t124 = *(uint64_t*)r_capacity;
    ((MpscQueue*)r_t119)->capacity = r_t124;
    return r_t119;
}

uint64_t enqueue(uint64_t r_arg_self, uint64_t r_arg_val) {
bb29:
    uint8_t r_self[8];
    *(uint64_t*)r_self = (uint64_t)r_arg_self;
    uint8_t r_val[8];
    *(uint64_t*)r_val = (uint64_t)r_arg_val;
bb30:
    uint8_t r_pos[8];
    uint64_t r_t125 = *(uint64_t*)r_self;
    uint64_t r_t126 = (uint64_t)((MpscQueue*)r_t125)->tail;
    uint64_t r_t128 = *(uint64_t*)r_self;
    uint64_t r_t127 = (uint64_t)&((MpscQueue*)r_t128)->tail;
    uint64_t r_t129 = atomic_load_explicit((_Atomic uint64_t*)r_t127, memory_order_relaxed);
    *(uint64_t*)r_pos = (uint64_t)r_t129;
    uint8_t r_s_ptr[8];
    uint64_t r_t130 = *(uint64_t*)r_self;
    uint64_t r_t131 = (uint64_t)((MpscQueue*)r_t130)->sequences;
    uint64_t r_t132 = *(uint64_t*)r_pos;
    uint64_t r_t133 = *(uint64_t*)r_self;
    uint64_t r_t134 = (uint64_t)((MpscQueue*)r_t133)->capacity;
    uint64_t r_t135 = (int64_t)r_t132 % (int64_t)r_t134;
    uint64_t r_t136 = 8;
    uint64_t r_t137 = r_t135 * r_t136;
    uint64_t r_t138 = r_t131 + r_t137;
    *(uint64_t*)r_s_ptr = (uint64_t)r_t138;
    uint8_t r_seq[8];
    uint64_t r_t139 = *(uint64_t*)r_s_ptr;
    uint64_t r_t140 = atomic_load_explicit((_Atomic uint64_t*)r_t139, memory_order_acquire);
    *(uint64_t*)r_seq = (uint64_t)r_t140;
    uint8_t r_diff[8];
    uint64_t r_t141 = *(uint64_t*)r_seq;
    uint64_t r_t142 = *(uint64_t*)r_pos;
    uint64_t r_t143 = r_t141 - r_t142;
    *(uint64_t*)r_diff = (uint64_t)r_t143;
    uint64_t r_t144 = *(uint64_t*)r_diff;
    uint64_t r_t145 = 0;
    uint64_t r_t146 = r_t144 == r_t145;
    if (r_t146) goto bb32; else goto bb33;
bb32:
    uint64_t r_t147 = *(uint64_t*)r_self;
    uint64_t r_t148 = (uint64_t)((MpscQueue*)r_t147)->tail;
    uint64_t r_t150 = *(uint64_t*)r_self;
    uint64_t r_t149 = (uint64_t)&((MpscQueue*)r_t150)->tail;
    uint64_t r_t151 = *(uint64_t*)r_pos;
    uint64_t r_t152 = *(uint64_t*)r_pos;
    uint64_t r_t153 = 1;
    uint64_t r_t154 = r_t152 + r_t153;
    uint64_t _exp_r_t155 = r_t151;
    uint64_t r_t155 = (uint64_t)atomic_compare_exchange_strong_explicit((_Atomic uint64_t*)r_t149, &_exp_r_t155, r_t154, memory_order_relaxed, memory_order_relaxed);
    if (r_t155) goto bb35; else goto bb36;
bb35:
    uint8_t r_d_ptr[8];
    uint64_t r_t156 = *(uint64_t*)r_self;
    uint64_t r_t157 = (uint64_t)((MpscQueue*)r_t156)->buffer;
    uint64_t r_t158 = *(uint64_t*)r_pos;
    uint64_t r_t159 = *(uint64_t*)r_self;
    uint64_t r_t160 = (uint64_t)((MpscQueue*)r_t159)->capacity;
    uint64_t r_t161 = (int64_t)r_t158 % (int64_t)r_t160;
    uint64_t r_t162 = 8;
    uint64_t r_t163 = r_t161 * r_t162;
    uint64_t r_t164 = r_t157 + r_t163;
    *(uint64_t*)r_d_ptr = (uint64_t)r_t164;
    // unsafe block begins
    uint64_t r_t165 = *(uint64_t*)r_val;
    uint64_t r_t166 = *(uint64_t*)r_d_ptr;
    uint64_t r_t167 = *(uint64_t*)r_d_ptr;
    *(uint64_t*)r_t167 = (uint64_t)r_t165;
    // unsafe block ends
    uint64_t r_t168 = *(uint64_t*)r_s_ptr;
    uint64_t r_t169 = *(uint64_t*)r_pos;
    uint64_t r_t170 = 1;
    uint64_t r_t171 = r_t169 + r_t170;
    atomic_store_explicit((_Atomic uint64_t*)r_t168, r_t171, memory_order_release);
    uint64_t r_t172 = 0;
    bool r_t173 = true;
    return r_t173;
    goto bb37;
bb36:
    goto bb37;
bb37:
    goto bb34;
bb33:
    uint64_t r_t174 = *(uint64_t*)r_diff;
    uint64_t r_t175 = 0;
    uint64_t r_t176 = (int64_t)r_t174 < (int64_t)r_t175;
    if (r_t176) goto bb38; else goto bb39;
bb38:
    bool r_t177 = false;
    return r_t177;
    goto bb40;
bb39:
    uint64_t r_t178 = (uint64_t)sched_yield();
    goto bb40;
bb40:
    goto bb34;
bb34:
    goto bb30;
bb31:
    return 0;
}

uint64_t dequeue(uint64_t r_arg_self) {
bb41:
    uint8_t r_self[8];
    *(uint64_t*)r_self = (uint64_t)r_arg_self;
bb42:
    uint8_t r_pos[8];
    uint64_t r_t179 = *(uint64_t*)r_self;
    uint64_t r_t180 = (uint64_t)((MpscQueue*)r_t179)->head;
    uint64_t r_t182 = *(uint64_t*)r_self;
    uint64_t r_t181 = (uint64_t)&((MpscQueue*)r_t182)->head;
    uint64_t r_t183 = atomic_load_explicit((_Atomic uint64_t*)r_t181, memory_order_relaxed);
    *(uint64_t*)r_pos = (uint64_t)r_t183;
    uint8_t r_s_ptr[8];
    uint64_t r_t184 = *(uint64_t*)r_self;
    uint64_t r_t185 = (uint64_t)((MpscQueue*)r_t184)->sequences;
    uint64_t r_t186 = *(uint64_t*)r_pos;
    uint64_t r_t187 = *(uint64_t*)r_self;
    uint64_t r_t188 = (uint64_t)((MpscQueue*)r_t187)->capacity;
    uint64_t r_t189 = (int64_t)r_t186 % (int64_t)r_t188;
    uint64_t r_t190 = 8;
    uint64_t r_t191 = r_t189 * r_t190;
    uint64_t r_t192 = r_t185 + r_t191;
    *(uint64_t*)r_s_ptr = (uint64_t)r_t192;
    uint8_t r_seq[8];
    uint64_t r_t193 = *(uint64_t*)r_s_ptr;
    uint64_t r_t194 = atomic_load_explicit((_Atomic uint64_t*)r_t193, memory_order_acquire);
    *(uint64_t*)r_seq = (uint64_t)r_t194;
    uint8_t r_diff[8];
    uint64_t r_t195 = *(uint64_t*)r_seq;
    uint64_t r_t196 = *(uint64_t*)r_pos;
    uint64_t r_t197 = 1;
    uint64_t r_t198 = r_t196 + r_t197;
    uint64_t r_t199 = r_t195 - r_t198;
    *(uint64_t*)r_diff = (uint64_t)r_t199;
    uint64_t r_t200 = *(uint64_t*)r_diff;
    uint64_t r_t201 = 0;
    uint64_t r_t202 = r_t200 == r_t201;
    if (r_t202) goto bb44; else goto bb45;
bb44:
    uint64_t r_t203 = *(uint64_t*)r_self;
    uint64_t r_t204 = (uint64_t)((MpscQueue*)r_t203)->head;
    uint64_t r_t206 = *(uint64_t*)r_self;
    uint64_t r_t205 = (uint64_t)&((MpscQueue*)r_t206)->head;
    uint64_t r_t207 = *(uint64_t*)r_pos;
    uint64_t r_t208 = *(uint64_t*)r_pos;
    uint64_t r_t209 = 1;
    uint64_t r_t210 = r_t208 + r_t209;
    uint64_t _exp_r_t211 = r_t207;
    uint64_t r_t211 = (uint64_t)atomic_compare_exchange_strong_explicit((_Atomic uint64_t*)r_t205, &_exp_r_t211, r_t210, memory_order_relaxed, memory_order_relaxed);
    if (r_t211) goto bb47; else goto bb48;
bb47:
    uint8_t r_d_ptr[8];
    uint64_t r_t212 = *(uint64_t*)r_self;
    uint64_t r_t213 = (uint64_t)((MpscQueue*)r_t212)->buffer;
    uint64_t r_t214 = *(uint64_t*)r_pos;
    uint64_t r_t215 = *(uint64_t*)r_self;
    uint64_t r_t216 = (uint64_t)((MpscQueue*)r_t215)->capacity;
    uint64_t r_t217 = (int64_t)r_t214 % (int64_t)r_t216;
    uint64_t r_t218 = 8;
    uint64_t r_t219 = r_t217 * r_t218;
    uint64_t r_t220 = r_t213 + r_t219;
    *(uint64_t*)r_d_ptr = (uint64_t)r_t220;
    uint8_t r_val[8];
    uint64_t r_t221 = 0;
    *(uint64_t*)r_val = (uint64_t)r_t221;
    // unsafe block begins
    uint64_t r_t222 = *(uint64_t*)r_d_ptr;
    uint64_t r_t223 = *(uint64_t*)r_t222;
    *(uint64_t*)r_val = (uint64_t)r_t223;
    // unsafe block ends
    uint64_t r_t224 = *(uint64_t*)r_s_ptr;
    uint64_t r_t225 = *(uint64_t*)r_pos;
    uint64_t r_t226 = *(uint64_t*)r_self;
    uint64_t r_t227 = (uint64_t)((MpscQueue*)r_t226)->capacity;
    uint64_t r_t228 = r_t225 + r_t227;
    atomic_store_explicit((_Atomic uint64_t*)r_t224, r_t228, memory_order_release);
    uint64_t r_t229 = 0;
    uint64_t r_t230 = *(uint64_t*)r_val;
    return r_t230;
    goto bb49;
bb48:
    goto bb49;
bb49:
    goto bb46;
bb45:
    uint64_t r_t231 = *(uint64_t*)r_diff;
    uint64_t r_t232 = 0;
    uint64_t r_t233 = (int64_t)r_t231 < (int64_t)r_t232;
    if (r_t233) goto bb50; else goto bb51;
bb50:
    uint64_t r_t234 = (uint64_t)sched_yield();
    goto bb52;
bb51:
    uint64_t r_t235 = (uint64_t)sched_yield();
    goto bb52;
bb52:
    goto bb46;
bb46:
    goto bb42;
bb43:
    return 0;
}

uint64_t _clos_tramp_0(uint64_t r___env) {
bb53:
    uint8_t r_q_ref[8];
    uint64_t r_t236 = (uint64_t)((_clos_env_0*)r___env)->q_ref;
    *(uint64_t*)r_q_ref = (uint64_t)r_t236;
    uint64_t r_t237 = *(uint64_t*)r_q_ref;
    uint64_t r_t238 = 1000;
    uint64_t r_t239 = 100;
    uint64_t r_t240 = (uint64_t)producer(r_t237, r_t238, r_t239);
    return 0;
}

uint64_t _clos_tramp_1(uint64_t r___env) {
bb54:
    uint8_t r_q_ref[8];
    uint64_t r_t241 = (uint64_t)((_clos_env_1*)r___env)->q_ref;
    *(uint64_t*)r_q_ref = (uint64_t)r_t241;
    uint64_t r_t242 = *(uint64_t*)r_q_ref;
    uint64_t r_t243 = 2000;
    uint64_t r_t244 = 100;
    uint64_t r_t245 = (uint64_t)producer(r_t242, r_t243, r_t244);
    return 0;
}

