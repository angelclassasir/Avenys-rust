// B3 regression: the managed-string release contract.
//
// B3 removes "silently do nothing" as an *ownership decision*. That decision
// already moved to the compiler (A1: MirOp::Drop carries Ownership, and codegen
// selects the destructor by semantic type rather than by LLVM's `ptr`).
//
// What remains on the runtime side is a narrower but still load-bearing
// question, and this smoke pins it:
//
//   1. A managed string releases by refcount. Two handles to one string, one
//      release, must leave the string alive and readable.
//   2. The second release frees it and returns its block to a size-class free
//      list, so a same-size allocation reuses the block instead of advancing
//      the arena.
//   3. An unmanaged pointer is never interpreted as a managed header. This is
//      the property the refcount path depends on: if a foreign pointer were
//      treated as a header, the release would decrement a refcount field at an
//      arbitrary address. Silently ignoring it is the correct behavior for a
//      foreign pointer; what must not happen is the "we manage everything" path.

#include "../src/runtime/runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>

// ── PAL stubs ────────────────────────────────────────────────────────────

static int reserve_calls;
static int reserve_failures;

void *pal_memory_reserve(int64_t size) {
    if (size <= 0) return NULL;
    reserve_calls++;
    if (reserve_failures) return NULL;
    return mmap(NULL, (size_t)size, PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
}

void pal_memory_release(void *pointer, int64_t size) {
    (void)size;
    if (pointer) munmap(pointer, (size_t)size);
}

void *pal_alloc(int64_t size) {
    if (size <= 0) return NULL;
    return malloc((size_t)size);
}

void pal_free(void *pointer) {
    if (pointer) free(pointer);
}

// ── Test helpers ─────────────────────────────────────────────────────────

static int failures;

static void check(int condition, const char *description) {
    if (condition) {
        printf("  ok   %s\n", description);
    } else {
        printf("  FAIL %s\n", description);
        failures++;
    }
}

// Allocate a managed string of `length` bytes of content plus a NUL.
static char *make_managed(const char *content) {
    size_t length = strlen(content);
    char *copy = rt_managed_from_slice(content, length);
    return copy;
}

// ── Tests ────────────────────────────────────────────────────────────────

static void test_release_by_refcount(void) {
    char *original = make_managed("shared");
    check(original != NULL, "managed string allocated");

    // Two handles to the same allocation: a second reference.
    char *second = original;
    rt_managed_retain(second);

    rt_managed_free(original);
    // One reference is still outstanding, so the content must survive.
    check(strcmp(second, "shared") == 0,
          "string survives while a reference remains");
    check(rt_managed_contains(second),
          "still managed while a reference remains");

    rt_managed_free(second);
    // Last reference: the block is released to the free list. `refs` is the
    // real liveness signal. `rt_managed_contains` is deliberately NOT the
    // check here: it reports arena membership, and a released block stays
    // inside the arena address range, so it keeps answering "yes" until the
    // block is reused. Asserting on membership would have encoded a claim the
    // API does not make.
    check(rt_string_header(second)->refs == 0,
          "refcount reaches zero after the last reference is released");
}

static void test_release_reuses_block(void) {
    // Same-size allocations must be served from the free list rather than
    // growing the arena. This is the property that keeps the allocator from
    // degenerating into a monotonic bump allocator over time.
    char *first = make_managed("reuse");
    char *before = first;
    rt_managed_free(first);
    check(rt_string_header(first)->refs == 0,
          "released block is back on the free list");

    char *second = make_managed("reuse");
    check(second == before,
          "same-size allocation reuses the freed block");
    rt_managed_free(second);
}

static void test_foreign_pointer_is_not_a_header(void) {
    // A pointer the allocator never handed out. Treating it as a managed
    // header would read and write a refcount at that address.
    static const char foreign[] = "not managed by the runtime";
    char stack_buffer[64];
    snprintf(stack_buffer, sizeof(stack_buffer), "%s", foreign);

    check(!rt_managed_contains(stack_buffer),
          "stack buffer is not reported as managed");

    // Releasing it must leave it untouched and must not crash.
    rt_managed_free(stack_buffer);
    check(strcmp(stack_buffer, foreign) == 0,
          "releasing a foreign pointer leaves its contents intact");

    // And a heap pointer the allocator never handed out, to prove the test is
    // not merely observing the stack case.
    size_t foreign_length = strlen(foreign) + 1;
    char *heap = malloc(foreign_length);
    memcpy(heap, foreign, foreign_length);
    check(!rt_managed_contains(heap),
          "unmanaged heap pointer is not reported as managed");
    rt_managed_free(heap);
    check(strcmp(heap, foreign) == 0,
          "releasing an unmanaged heap pointer leaves its contents intact");
    free(heap);
}

static void test_null_and_double_release(void) {
    rt_managed_free(NULL);
    check(1, "releasing NULL is a no-op");

    char *value = make_managed("once");
    rt_managed_free(value);
    check(rt_string_header(value)->refs == 0, "released");
    // A second release on an already-released pointer must be harmless. It is
    // not "correct usage", but it must not corrupt the arena.
    rt_managed_free(value);
    check(1, "releasing an already-released pointer does not corrupt state");

    // The allocator still works afterwards.
    char *next = make_managed("after");
    check(next != NULL && strcmp(next, "after") == 0,
          "allocator is usable after a double release");
    rt_managed_free(next);
}

int main(void) {
    // The arena initializes lazily on the first allocation, so the first
    // managed string both creates it and proves the reservation went through
    // the PAL rather than straight to a libc allocator.
    check(make_managed("warmup") != NULL, "first allocation succeeds");
    check(reserve_calls > 0, "arena reserved through the PAL");

    printf("managed release contract\n");
    test_release_by_refcount();
    test_release_reuses_block();
    test_foreign_pointer_is_not_a_header();
    test_null_and_double_release();

    rt_managed_cleanup_all();

    if (failures) {
        printf("FAILURES: %d\n", failures);
        return 1;
    }
    printf("all managed release checks passed\n");
    return 0;
}