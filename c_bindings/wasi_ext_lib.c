#include <wasi/api.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>

#include "wasi_ext_lib.h"

#define SYSCALL_LENGTH 256
#define SYSCALL_ARGS_LENGTH 2048

int __syscall(const char *command, char *args, uint8_t *output_buf, size_t output_buf_len) {
    char c[SYSCALL_LENGTH];
    sprintf(c, "/!{\"command\": \"%s\", \"buf_len\": %ld, \"buf_ptr\": %p}", command, strlen(args), args);

    size_t written;
    int err = __wasi_path_readlink(3, c, output_buf, output_buf_len, &written);
    return err;
}

int wasi_ext_chdir(const char *path) {
    // wasilib doesn't support realpath, so the given path must be canonicalized
    char args[SYSCALL_ARGS_LENGTH];
    sprintf(args, "{\"dir\": \"%s\"}", path);
    return __syscall("chdir", args, NULL, 0);
}

int wasi_ext_getcwd(char *path, size_t buf_len) {
    char c[SYSCALL_ARGS_LENGTH];
    sprintf(c, "/!{\"buf\": %p, \"buf_len\": %zu}", path, buf_len);
    return __syscall("getcwd", c, (uint8_t*)path, buf_len);
}

int wasi_ext_isatty(int fd) {
    char args[SYSCALL_ARGS_LENGTH];
    const size_t output_len = 64;
    char output[output_len];
    sprintf(args, "{ \"fd\": %d }", fd);
    int err = __syscall("isatty", args, (uint8_t*)output, output_len);
    if (err != 0) { return -err; }
    return atoi(output);
}

int wasi_ext_set_env(const char *attrib, const char *val) {
    char args[SYSCALL_ARGS_LENGTH];
    if (val != NULL) {
        sprintf(args, "{ \"attrib\": %s, \"val\": %s }", attrib, val);
    } else {
        sprintf(args, "{ \"attrib\": %s }", attrib);
    }
    return __syscall("set_env", args, NULL, 0);
}
