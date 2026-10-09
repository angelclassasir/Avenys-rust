#include "../src/pal/pal.h"

#include <errno.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

typedef struct {
    char socket_path[108];
    int result;
} client_context_t;

static int send_all(pal_socket_t socket_handle, const char *bytes, size_t length) {
    size_t offset = 0;
    while (offset < length) {
        int64_t written = pal_socket_send(socket_handle, bytes + offset,
                                          (int64_t)(length - offset));
        if (written <= 0) return 0;
        offset += (size_t)written;
    }
    return 1;
}

static int receive_all(pal_socket_t socket_handle, char *bytes, size_t length) {
    size_t offset = 0;
    while (offset < length) {
        int64_t received = pal_socket_recv(socket_handle, bytes + offset,
                                           (int64_t)(length - offset));
        if (received <= 0) return 0;
        offset += (size_t)received;
    }
    return 1;
}

static void *run_client(void *context_pointer) {
    client_context_t *context = context_pointer;
    pal_socket_t socket_handle = pal_unix_connect(
        context->socket_path, PAL_SOCKET_UNIX_STREAM);
    if (socket_handle.index == 0 && socket_handle.generation == 0) {
        context->result = 1;
        return NULL;
    }

    char response[5] = {0};
    context->result = send_all(socket_handle, "hello", 5) &&
                      receive_all(socket_handle, response, 5) &&
                      memcmp(response, "world", 5) == 0 ? 0 : 2;
    pal_socket_close(socket_handle);
    return NULL;
}

int main(void) {
    char private_directory[] = "/tmp/avenys-pal-unix-XXXXXX";
    if (!mkdtemp(private_directory)) return 1;

    client_context_t client = {0};
    if (snprintf(client.socket_path, sizeof(client.socket_path), "%s/channel.sock",
                 private_directory) >= (int)sizeof(client.socket_path)) return 2;

    pal_listener_t listener = pal_unix_bind(
        client.socket_path, PAL_SOCKET_UNIX_STREAM);
    if (listener.index == 0 && listener.generation == 0) {
        fprintf(stderr, "Unix socket bind failed: %s (%d)\n",
                pal_last_error_message(), (int)pal_last_error());
        return 3;
    }

    pthread_t client_thread;
    if (pthread_create(&client_thread, NULL, run_client, &client) != 0) return 4;

    pal_socket_t accepted = pal_listener_accept(listener);
    if (accepted.index == 0 && accepted.generation == 0) return 5;
    char request[5] = {0};
    int exchange_ok = receive_all(accepted, request, 5) &&
                      memcmp(request, "hello", 5) == 0 &&
                      send_all(accepted, "world", 5);
    pal_socket_close(accepted);

    if (pthread_join(client_thread, NULL) != 0) return 6;
    pal_listener_close(listener);

    struct stat socket_status;
    int socket_removed = lstat(client.socket_path, &socket_status) != 0 && errno == ENOENT;

    pal_listener_t insecure = pal_unix_bind(
        "/tmp/avenys-pal-unix-insecure.sock", PAL_SOCKET_UNIX_STREAM);
    int insecure_path_rejected = insecure.index == 0 && insecure.generation == 0 &&
                                 pal_last_error() == PAL_ERR_PERMISSION;
    char missing_socket_path[108];
    snprintf(missing_socket_path, sizeof(missing_socket_path), "%s/missing.sock",
             private_directory);
    pal_socket_t missing = pal_unix_connect(
        missing_socket_path, PAL_SOCKET_UNIX_STREAM);
    int missing_path_reported = missing.index == 0 && missing.generation == 0 &&
                                pal_last_error() == PAL_ERR_NOT_FOUND;
    if (!(exchange_ok && client.result == 0 && socket_removed &&
          insecure_path_rejected && missing_path_reported)) {
        fprintf(stderr,
                "Unix PAL smoke failed: exchange=%d client=%d socket_removed=%d "
                "insecure_rejected=%d missing_path_reported=%d last_error=%d\n",
                exchange_ok, client.result, socket_removed, insecure_path_rejected,
                missing_path_reported,
                (int)pal_last_error());
        rmdir(private_directory);
        return 7;
    }

    if (rmdir(private_directory) != 0) return 8;
    puts("PAL Unix stream socket: thread exchange and private-path checks passed");
    return 0;
}
