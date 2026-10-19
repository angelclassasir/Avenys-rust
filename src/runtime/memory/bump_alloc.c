#include "runtime.h"
#include "pal.h"
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <stdatomic.h>

// Fast thread-local bump allocator with size-class arenas.
// Each thread gets its own arenas to avoid contention.
// Size classes: 32, 64, 128, 256, 512, 1024, 4096, 16384+ bytes.

#define NUM_ARENAS 8
#define ARENA_SIZE (2 * 1024 * 1024)  // 2MB per arena

// Size classes: 32, 64, 128, 256, 512, 1024, 4096, 16384+ bytes.
static const size_t SIZE_CLASSES[8] = {32, 64, 128, 256, 512, 1024, 4096, 16384};

static __thread char *arena_bases[8] = {NULL};
static __thread char *arena_ptrs[8] = {NULL};
static __thread char *arena_ends[8] = {NULL};
static _Atomic(int) arena_initialized = 0;

static void arena_ensure_initialized(void) {
    if (atomic_load(&arena_initialized)) return;
    if (atomic_exchange(&arena_initialized, 1)) return;
    
    for (int i = 0; i < 8; i++) {
        size_t size = (i == 7) ? (4 * 1024 * 1024) : (SIZE_CLASSES[i] * 1024); // Last class gets 4MB
        char *base = (char *)pal_memory_reserve(ARENA_SIZE);
        if (!base) return;
        if (!pal_memory_commit(base, 4096)) {
            pal_memory_release(base, ARENA_SIZE);
            return;
        }
        // Store arena base in first 8 bytes
        *(uint64_t*)base = 0; // bump_ptr = 0
        arena_bases[i] = base;
        arena_ptrs[i] = base;
        arena_ends[i] = base + ARENA_SIZE;
    }
    atomic_store(&arena_initialized, 1);
}

static inline int size_class_index(size_t size) {
    if (size <= 32) return 0;
    if (size <= 64) return 1;
    if (size <= 128) return 2;
    if (size <= 256) return 3;
    if (size <= 512) return 4;
    if (size <= 1024) return 4;
    if (size <= 4096) return 5;
    if (size <= 16384) return 6;
    return 7;
}

static inline void arena_ensure_initialized(void) {
    if (!arena_initialized) {
        arena_ensure_initialized();
    }
}

static inline void *arena_alloc_fast(int class_idx, size_t size) {
    if (!arena_initialized) arena_ensure_initialized();
    
    char *ptr = arena_ptrs[class_idx];
    char *end = arena_ends[class_idx];
    char *new_ptr = arena_ptrs[class_idx] + size;
    
    if ((uintptr_t)new_ptr <= (uintptr_t)arena_ends[class_idx]) {
        arena_ptrs[class_idx] = new_ptr;
        return arena_ptrs[class_idx] - size;
    }
    return NULL;
}

static void *arena_alloc_slow(int class_idx, size_t size) {
    // Try to expand current arena or create new one
    size_t new_size = ARENA_SIZE;
    char *base = (char *)pal_memory_reserve(ARENA_SIZE);
    if (!base) return NULL;
    if (!pal_memory_commit(base, 4096)) {
        pal_memory_release(base, ARENA_SIZE);
        return NULL;
    }
    // For simplicity, we'll just return NULL and let caller handle
    // In production, we'd chain arenas or grow the existing one
    return NULL;
}

// Public API

void *bump_alloc(size_t size) {
    if (!arena_initialized) arena_ensure_initialized();
    
    int class_idx = size_class_index(size);
    if (class_idx >= 8) class_idx = 7;
    
    void *ptr = arena_alloc_fast(class_idx, size);
    if (ptr) return ptr;
    
    // Slow path - try to expand or get new arena
    void *ptr = arena_alloc_slow(class_idx, size);
    if (ptr) return ptr;
    
    // Fallback to system allocator
    return pal_memory_alloc(size);
}

void *bump_alloc_class(size_t size, int class_idx) {
    if (!arena_initialized) arena_ensure_initialized();
    if (class_idx >= 8) class_idx = 7;
    
    void *ptr = arena_alloc_fast(class_idx, size);
    if (ptr) return ptr;
    return NULL;
}

void *bump_realloc(void *ptr, size_t old_size, size_t new_size) {
    if (!ptr) return bump_alloc(new_size);
    if (new_size <= 0) {
        // Free not needed for bump allocator
        return NULL;
    }
    
    // For now, allocate new and copy
    void *new_ptr = bump_alloc(new_size);
    if (!new_ptr) return NULL;
    
    // Copy old data (we don't track actual size, use minimum)
    size_t copy_size = old_size < new_size ? old_size : new_size;
    memcpy(bump_ptr_get(ptr), ptr, copy_size);
    return new_ptr;
}

void bump_free(void *ptr) {
    // No-op for bump allocator - memory reclaimed on reset
}

void *bump_ptr_get(void) {
    return arena_ptrs[0];
}

void *bump_end_get(void) {
    return arena_ends[0];
}

void bump_reset(void) {
    if (!arena_initialized) return;
    for (int i = 0; i < 8; i++) {
        if (arena_bases[i]) {
            arena_ptrs[i] = arena_bases[i];
        }
    }
}

void bump_init(void) {
    if (arena_initialized) return;
    arena_ensure_initialized();
}

void *bump_arena_new(size_t size) {
    size_t size_rounded = size;
    if (size < 64*1024) size = 64*1024;
    if (size > 2*1024*1024*1024) size = 2*1024*1024*1024;
    
    char *base = (char *)pal_memory_reserve(size);
    if (!base) return NULL;
    if (!pal_memory_commit(base, 4096)) {
        pal_memory_release(base, size);
        return NULL;
    }
    
    // Store metadata at start of arena
    *(uint64_t*)base = 0;  // bump_ptr = 0
    *(uint64_t*)(base + 8) = size; // limit
    
    char *base_ptr = base + 16; // Skip header
    for (int i = 0; i < 8; i++) {
        if (!arena_bases[i]) {
            arena_bases[i] = base;
            arena_ptrs[i] = base_ptr;
            arena_ends[i] = base + size;
        }
    }
    return base + 16;
}

void *bump_alloc_class(size_t size, int class_idx) {
    if (!arena_initialized) arena_ensure_initialized();
    if (class_idx >= 8) class_idx = 7;
    return arena_alloc_fast(class_idx, size);
}

void bump_free(void *ptr) {
    // No-op for bump allocator - memory reclaimed on reset
}

void *bump_ptr_get(void) {
    return arena_ptrs[0];
}

void *bump_end_get(void) {
    return arena_ends[0];
}

void bump_reset(void) {
    if (!arena_initialized) return;
    for (int i = 0; i < 8; i++) {
        if (arena_bases[i]) {
            arena_ptrs[i] = arena_bases[i];
        }
    }
}

void bump_init(void) {
    if (arena_initialized) return;
    arena_ensure_initialized();
}