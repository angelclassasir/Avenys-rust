#include "../src/pal/pal.h"
#include "../src/runtime/runtime.h"

#include <stdint.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/wait.h>
#include <unistd.h>

typedef struct {
    const char *const *items;
    int64_t length;
} test_argument_list_t;

char **rt_build_argv(const char *command, void *arguments, int64_t *argument_count) {
    if (!command || !arguments || !argument_count) return NULL;
    test_argument_list_t *list = arguments;
    if (list->length < 0 || list->length > (INT64_MAX / (int64_t)sizeof(char *)) - 2)
        return NULL;
    char **argv = calloc((size_t)list->length + 2, sizeof(char *));
    if (!argv) return NULL;
    argv[0] = strdup(command);
    if (!argv[0]) { free(argv); return NULL; }
    for (int64_t index = 0; index < list->length; index++) {
        argv[index + 1] = strdup(list->items[index]);
        if (!argv[index + 1]) {
            rt_free_argv(argv, index + 1);
            return NULL;
        }
    }
    *argument_count = list->length + 1;
    return argv;
}

void rt_free_argv(char **argv, int64_t argument_count) {
    if (!argv) return;
    for (int64_t index = 0; index < argument_count; index++) free(argv[index]);
    free(argv);
}

int main(void) {
    char working_directory[] = "/tmp/avenys-runtime-process-XXXXXX";
    if (!mkdtemp(working_directory)) return 1;
    pal_sbx_set_root(working_directory);

    const char *printf_arguments[] = {"proc-ok"};
    test_argument_list_t printf_list = {printf_arguments, 1};
    char *output = rt_proc_capture_argv("/usr/bin/printf", &printf_list);
    int basic_capture_ok = output && strcmp(output, "proc-ok") == 0 &&
                           rt_proc_last_exit() == 0;

    char *directory_output = rt_proc_capture_argv2(
        "/bin/pwd", &(test_argument_list_t){NULL, 0}, working_directory, 0);
    size_t directory_length = strlen(working_directory);
    int directory_ok = directory_output &&
                       strncmp(directory_output, working_directory, directory_length) == 0 &&
                       directory_output[directory_length] == '\n' &&
                       rt_proc_last_exit() == 0;

    const char *merge_arguments[] = {"-c", "printf out; printf err >&2"};
    test_argument_list_t merge_list = {merge_arguments, 2};
    char *merged_output = rt_proc_capture_argv2("/bin/sh", &merge_list,
                                                 working_directory, 1);
    int merge_ok = merged_output && strcmp(merged_output, "outerr") == 0 &&
                   rt_proc_last_exit() == 0;

    const char *exit_arguments[] = {"-c", "exit 7"};
    test_argument_list_t exit_list = {exit_arguments, 2};
    char *empty_output = rt_proc_capture_argv2("/bin/sh", &exit_list,
                                                working_directory, 0);
    int exit_code_ok = empty_output && empty_output[0] == '\0' &&
                       rt_proc_last_exit() == 7;

    const char *no_arguments[] = {NULL};
    test_argument_list_t no_argument_list = {no_arguments, 0};
    char missing_directory[256];
    snprintf(missing_directory, sizeof(missing_directory), "%s/missing", working_directory);
    char *bad_directory_output = rt_proc_capture_argv2(
        "/bin/pwd", &no_argument_list, missing_directory, 0);
    int bad_directory_ok = bad_directory_output && bad_directory_output[0] == '\0' &&
                           rt_proc_last_exit() == 126;

    const char *signal_arguments[] = {"-c", "kill -TERM $$"};
    test_argument_list_t signal_list = {signal_arguments, 2};
    char *signal_output = rt_proc_capture_argv2("/bin/sh", &signal_list,
                                                 working_directory, 0);
    int signal_status_ok = signal_output && signal_output[0] == '\0' &&
                           rt_proc_last_exit() == 128 + SIGTERM;

    rt_managed_cleanup_all();
    rmdir(working_directory);
    if (!(basic_capture_ok && directory_ok && merge_ok && exit_code_ok &&
          bad_directory_ok && signal_status_ok)) {
        fprintf(stderr,
                "Runtime PAL process smoke failed: capture=%d cwd=%d merge=%d exit=%d cwd_error=%d signal=%d\n",
                basic_capture_ok, directory_ok, merge_ok, exit_code_ok,
                bad_directory_ok, signal_status_ok);
        return 2;
    }
    puts("Runtime argv process capture, working directory, merged stderr, and exit statuses passed through PAL.");
    return 0;
}
