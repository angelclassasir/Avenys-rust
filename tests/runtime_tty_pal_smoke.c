#include "../src/pal/pal.h"
#include "../src/runtime/runtime.h"

#include <pty.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

static int verify_controlling_terminal_line(void) {
    int result_pipe[2];
    if (pipe(result_pipe) != 0) return 0;

    int terminal_master = -1;
    pid_t child = forkpty(&terminal_master, NULL, NULL, NULL);
    if (child < 0) return 0;
    if (child == 0) {
        close(result_pipe[0]);
        char *line = rt_read_tty();
        size_t length = strlen(line);
        int result = length == strlen("confirm") &&
                     memcmp(line, "confirm", length) == 0 ? 1 :
                     strcmp(line, "y") == 0 ? 2 : 3;
        (void)write(result_pipe[1], &result, sizeof(result));
        _exit(result == 1 ? 0 : 1);
    }

    close(result_pipe[1]);
    const char input[] = "confirm\n";
    int sent = write(terminal_master, input, sizeof(input) - 1) == sizeof(input) - 1;
    int child_result = 0;
    ssize_t received = read(result_pipe[0], &child_result, sizeof(child_result));
    close(result_pipe[0]);
    int status = 0;
    if (waitpid(child, &status, 0) != child) {
        close(terminal_master);
        return 0;
    }
    close(terminal_master);
    int succeeded = sent && received == sizeof(child_result) && child_result == 1 &&
                    WIFEXITED(status) && WEXITSTATUS(status) == 0;
    if (!succeeded) {
        fprintf(stderr, "PTY read diagnostics: sent=%d received=%zd result=%d status=%d\n",
                sent, received, child_result, status);
    }
    return succeeded;
}

static int verify_missing_terminal_fallback(void) {
    int result_pipe[2];
    if (pipe(result_pipe) != 0) return 0;
    pid_t child = fork();
    if (child < 0) return 0;
    if (child == 0) {
        close(result_pipe[0]);
        if (setsid() < 0) _exit(2);
        char *line = rt_read_tty();
        int succeeded = strcmp(line, "y") == 0;
        (void)write(result_pipe[1], &succeeded, sizeof(succeeded));
        _exit(succeeded ? 0 : 1);
    }

    close(result_pipe[1]);
    int child_result = 0;
    ssize_t received = read(result_pipe[0], &child_result, sizeof(child_result));
    close(result_pipe[0]);
    int status = 0;
    if (waitpid(child, &status, 0) != child) return 0;
    return received == sizeof(child_result) && child_result == 1 &&
           WIFEXITED(status) && WEXITSTATUS(status) == 0;
}

int main(void) {
    if (!verify_controlling_terminal_line()) {
        fputs("Controlling-terminal line read failed.\n", stderr);
        return 1;
    }
    if (!verify_missing_terminal_fallback()) {
        fputs("Missing-terminal fallback failed.\n", stderr);
        return 2;
    }
    puts("Runtime controlling-terminal PAL read and non-interactive fallback passed.");
    return 0;
}
