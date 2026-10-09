#include "runtime.h"
#include "pal.h"
#include <stdlib.h>

static void* closure_thread_start(void* arg) {
    void** closure = (void**)arg;
    int64_t (*fn)(void*) = (int64_t (*)(void*))closure[0];
    void* env = closure[1];
    free(arg);
    int64_t result = fn(env);
    return (void*)result;
}

int64_t rt_thread_spawn_closure(void* fn_ptr, void* env_ptr) {
    void** closure = (void**)malloc(sizeof(void*) * 2);
    if (!closure) return -1;
    closure[0] = fn_ptr;
    closure[1] = env_ptr;
    return pal_thread_spawn(closure_thread_start, closure);
}
