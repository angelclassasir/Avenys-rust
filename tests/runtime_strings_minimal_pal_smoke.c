#include "../src/runtime/runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int outstanding_allocations;

int64_t rt_strings_len(const char *value) {
    return value ? (int64_t)strlen(value) : 0;
}

void *pal_alloc(int64_t size) {
    if (size <= 0) return NULL;
    void *pointer = malloc((size_t)size);
    if (pointer) outstanding_allocations++;
    return pointer;
}

void *pal_realloc(void *pointer, int64_t size) {
    if (size <= 0) return NULL;
    int was_unallocated = pointer == NULL;
    void *resized = realloc(pointer, (size_t)size);
    if (resized && was_unallocated) outstanding_allocations++;
    return resized;
}

void pal_free(void *pointer) {
    if (pointer) outstanding_allocations--;
    free(pointer);
}

static int expect_string(char *actual, const char *expected) {
    if (!actual || strcmp(actual, expected) != 0) {
        fprintf(stderr, "expected <%s>, got <%s>\n", expected,
                actual ? actual : "<null>");
        if (actual) pal_free(actual);
        return 0;
    }
    pal_free(actual);
    return 1;
}

int main(void) {
    if (!expect_string(rt_string_copy("copy"), "copy") ||
        !expect_string(rt_string_concat("two ", "words"), "two words") ||
        !expect_string(rt_strings_repeat("x", 3), "xxx") ||
        !expect_string(rt_strings_replace("abcabc", "abc", "long"), "longlong") ||
        !expect_string(rt_strings_pad_left("x", 3, "0"), "00x") ||
        !expect_string(rt_strings_pad_right("x", 3, "-"), "x--") ||
        !expect_string(rt_strings_trim(" \tx \n"), "x") ||
        !expect_string(rt_strings_substr("abcdef", 2, 3), "cde")) {
        return 1;
    }

    char *joined_parts[] = {"one", "two"};
    if (!expect_string(rt_strings_join(joined_parts, 2, "-"), "one-two")) {
        return 1;
    }

    char **arguments = (char **)pal_alloc(2 * (int64_t)sizeof(char *));
    if (!arguments) return 1;
    arguments[0] = rt_string_copy("program");
    arguments[1] = rt_string_copy("argument");
    rt_free_argv(arguments, 2);

    if (outstanding_allocations != 0) {
        fprintf(stderr, "minimal string PAL allocations remain: %d\n",
                outstanding_allocations);
        return 1;
    }

    puts("Minimal string operations use matched PAL allocation and release.");
    return 0;
}
