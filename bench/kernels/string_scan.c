/* A7: substring search over a large text.
 *
 * Builds a long repeated text once and scans it for a needle repeatedly. The C
 * leg uses strstr on the same text so both legs perform identical searches.
 */
#include "_common.h"
#include <string.h>

int main(void) {
    int64_t seed = SEED;
    const char *unit = "the quick brown fox jumps over the lazy dog. ";
    size_t unit_len = strlen(unit);
    size_t n = 20000;
    char *haystack = (char *)malloc((n * unit_len) + 1);
    if (!haystack) return 1;
    char *p = haystack;
    for (size_t i = 0; i < n; i++) {
        memcpy(p, unit, unit_len);
        p += unit_len;
    }
    *p = '\0';
    const char *needle = "lazy dog";
    for (int64_t round = 0; round < 20; round++) {
        int64_t hits = 0;
        const char *found = strstr(haystack, needle);
        while (found) {
            hits++;
            found = strstr(found + 1, needle);
        }
        seed = (seed + hits) % 1000003;
    }
    free(haystack);
    printf("%lld\n", (long long)seed);
    return 0;
}
