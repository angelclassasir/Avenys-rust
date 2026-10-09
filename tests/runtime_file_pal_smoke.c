#include "../src/pal/pal.h"
#include "../src/runtime/runtime.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static int write_fixture(const char *path, const void *bytes, size_t length) {
    FILE *file = fopen(path, "wb");
    if (!file) return 0;
    int ok = fwrite(bytes, 1, length, file) == length;
    return fclose(file) == 0 && ok;
}

static int read_fixture(const char *path, void *bytes, size_t length) {
    FILE *file = fopen(path, "rb");
    if (!file) return 0;
    int ok = fread(bytes, 1, length, file) == length;
    return fclose(file) == 0 && ok;
}

int main(void) {
    char directory[] = "/tmp/avenys-runtime-file-pal-XXXXXX";
    if (!mkdtemp(directory)) return 1;

    char input_path[256];
    char output_path[256];
    char log_path[256];
    snprintf(input_path, sizeof(input_path), "%s/input.txt", directory);
    snprintf(output_path, sizeof(output_path), "%s/output.bin", directory);
    snprintf(log_path, sizeof(log_path), "%s/events.log", directory);
    if (!write_fixture(input_path, "abc", 3)) return 2;
    pal_sbx_set_root(directory);

    char *contents = rt_read_bytes(input_path);
    int read_ok = contents && strcmp(contents, "abc") == 0;
    int read_error = (int)pal_last_error();
    char read_error_message[128];
    snprintf(read_error_message, sizeof(read_error_message), "%s", pal_last_error_message());
    pal_free(contents);
    char *relative_contents = rt_read_bytes("input.txt");
    int relative_read_ok = relative_contents && strcmp(relative_contents, "abc") == 0;
    pal_free(relative_contents);
    char *digest = rt_crypto_sha256_file_hex(input_path);
    int digest_ok = digest && strcmp(
        digest, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad") == 0;
    int digest_error = (int)pal_last_error();
    char digest_error_message[128];
    snprintf(digest_error_message, sizeof(digest_error_message), "%s",
             pal_last_error_message());
    char *base64 = rt_crypto_base64_file(input_path);
    int base64_ok = base64 && strcmp(base64, "YWJj") == 0;
    rt_managed_free(base64);
    int malformed_base64_rejected =
        !rt_crypto_ed25519_verify_b64("!", input_path, "!");

    int write_ok = rt_hex_to_file(output_path, "4100ff");
    unsigned char binary[3] = {0};
    int binary_ok = read_fixture(output_path, binary, sizeof(binary)) &&
                    binary[0] == 0x41 && binary[1] == 0x00 && binary[2] == 0xff;

    int log_first = rt_web_log_append(log_path, "first event");
    int log_second = rt_web_log_append(log_path, "second event");
    char log_contents[64] = {0};
    FILE *log_file = fopen(log_path, "rb");
    size_t log_length = log_file ? fread(log_contents, 1, sizeof(log_contents) - 1, log_file) : 0;
    if (log_file) fclose(log_file);
    int log_ok = log_first == 0 && log_second == 0 &&
                 log_length == strlen("first event\nsecond event\n") &&
                 strcmp(log_contents, "first event\nsecond event\n") == 0;

    int traversal_denied = rt_read_bytes("../outside-file") == NULL &&
                           pal_last_error() == PAL_ERR_PERMISSION;

    if (!read_ok || !digest_ok) {
        fprintf(stderr, "Read/hash PAL errors: read=%d (%s), digest=%d (%s)\n",
                read_error, read_error_message, digest_error, digest_error_message);
    }

    rt_managed_cleanup_all();
    unlink(input_path);
    unlink(output_path);
    unlink(log_path);
    rmdir(directory);

    if (!(read_ok && relative_read_ok && digest_ok && base64_ok &&
          malformed_base64_rejected && write_ok && binary_ok && log_ok &&
          traversal_denied)) {
        fprintf(stderr,
                "Runtime file PAL smoke failed: read=%d relative_read=%d digest=%d base64=%d malformed=%d write=%d binary=%d log=%d traversal=%d\n",
                read_ok, relative_read_ok, digest_ok, base64_ok,
                malformed_base64_rejected, write_ok, binary_ok, log_ok,
                traversal_denied);
        return 3;
    }

    puts("Runtime whole-file reads, hashing, binary writes, append, and sandbox rejection passed through PAL.");
    return 0;
}
