#include "../src/runtime/runtime.h"

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

static int64_t allocation_calls;
static int64_t release_calls;

void *pal_alloc(int64_t size) {
    if (size <= 0) return NULL;
    allocation_calls++;
    return malloc((size_t)size);
}

void pal_free(void *pointer) {
    if (pointer) release_calls++;
    free(pointer);
}

int main(void) {
    void *maybe = rt_maybe_some_i64(42);
    int maybe_ok = maybe && rt_maybe_unwrap_or_i64(maybe, -1) == 42;
    rt_maybe_free(maybe);

    void *success = rt_result_ok_i64(17);
    int success_ok = success && rt_result_is_ok(success) &&
                     rt_result_unwrap_or_i64(success, -1) == 17;
    rt_result_free(success);

    void *failure = rt_result_err_i64(9);
    int failure_ok = failure && rt_result_is_err(failure) &&
                     rt_result_unwrap_or_i64(failure, 23) == 23;
    rt_result_free(failure);

    if (!maybe_ok || !success_ok || !failure_ok ||
        allocation_calls != 3 || release_calls != 3) {
        fprintf(stderr,
                "Maybe/Result PAL smoke failed: maybe=%d ok=%d err=%d alloc=%lld free=%lld\n",
                maybe_ok, success_ok, failure_ok,
                (long long)allocation_calls, (long long)release_calls);
        return 1;
    }
    puts("Maybe and Result descriptors preserve behavior and use PAL allocation.");
    return 0;
}
