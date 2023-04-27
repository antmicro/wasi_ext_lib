/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#include <wasi/api.h>

#include <errno.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "wasi_ext_lib.h"

#include <json/json.h>

#define SYSCALL_LENGTH 256
#define SYSCALL_ARGS_LENGTH 2048

JsonNode *json_mkredirect(struct Redirect redir) {
    JsonNode *node = json_mkobject();
    json_append_member(node, "fd", json_mknumber((double)redir.fd));
    json_append_member(node, "path", json_mkstring(redir.path));
    switch (redir.type) {
    case READ:
        json_append_member(node, "mode", json_mkstring("read"));
        break;
    case WRITE:
        json_append_member(node, "mode", json_mkstring("write"));
        break;
    case APPEND:
        json_append_member(node, "mode", json_mkstring("append"));
        break;
    }
    return node;
}

int __syscall(const char *command, char *args, uint8_t *output_buf,
              size_t output_buf_len) {
    char *ptr;
    asprintf(&ptr, "%p", args);
    JsonNode *root = json_mkobject();
    json_append_member(root, "command", json_mkstring(command));
    json_append_member(root, "buf_len", json_mknumber((double)strlen(args)));
    json_append_member(root, "buf_ptr", json_mkstring(ptr));

    char *serialized = json_stringify(1, root, " ");
    json_delete(root);

    size_t written;
    int err = __wasi_path_readlink(3, serialized, output_buf, output_buf_len,
                                   &written);
    free(ptr);
    free(serialized);
    return err;
}

int wasi_ext_chdir(const char *path) {
    // wasi lib doesn't support realpath, so the given path must be
    // canonicalized
    JsonNode *root = json_mkobject();
    json_append_member(root, "dir", json_mkstring(path));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("chdir", serialized, NULL, 0);
    free(serialized);
    return err;
}

int wasi_ext_getcwd(char *path, size_t buf_len) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "buf_len", json_mknumber((double)buf_len));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("getcwd", serialized, (uint8_t *)path, buf_len);
    free(serialized);
    return err;
}

int wasi_ext_isatty(int fd) {
    const size_t output_len = 64;
    char output[output_len];

    JsonNode *root = json_mkobject();
    json_append_member(root, "fd", json_mknumber((double)fd));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("isatty", serialized, (uint8_t *)output, output_len);
    free(serialized);
    if (err != 0) {
        return -err;
    }
    return atoi(output);
}

int wasi_ext_set_env(const char *attrib, const char *val) {
    if (val == NULL) {
        if (unsetenv(attrib) != 0) {
            return errno;
        }
    } else {
        if (setenv(attrib, val, 1) != 0) {
            return errno;
        }
    }
    JsonNode *root = json_mkobject();
    json_append_member(root, "key", json_mkstring(attrib));
    if (val != NULL) {
        json_append_member(root, "value", json_mkstring(val));
    }

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("set_env", serialized, NULL, 0);
    free(serialized);
    return err;
}

int wasi_ext_getpid() {
    char args[] = "{}";
    const size_t output_len = 16;
    char output[output_len];
    int result = __syscall("getpid", args, (uint8_t *)output, output_len);
    if (result != 0) {
        return -result;
    } else {
        return atoi(output);
    }
}

int wasi_ext_set_echo(int should_echo) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "echo", json_mkbool(should_echo == 1));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("set_echo", serialized, NULL, 0);
    free(serialized);
    return err;
}

#ifdef HTERM
int wasi_ext_hterm_set(const char *attrib, const char *val) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "method", json_mkstring("set"));
    json_append_member(root, "attrib", json_mkstring(attrib));
    json_append_member(root, "val", json_mkstring(val));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("hterm", serialized, NULL, 0);
    free(serialized);
    return err;
}

int wasi_ext_hterm_get(const char *attrib, char *val, size_t val_len) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "method", json_mkstring("get"));
    json_append_member(root, "attrib", json_mkstring(attrib));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("hterm", serialized, (uint8_t *)val, val_len);
    free(serialized);
    return err;
}

int wasi_ext_event_source_fd(uint32_t event_mask) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "event_mask", json_mknumber(event_mask));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    const size_t output_len = 16;
    char output[output_len];

    int err =
        __syscall("event_source_fd", serialized, (uint8_t *)output, output_len);
    free(serialized);
    if (err != 0) {
        return -err;
    }
    return atoi(output);
}

int wasi_ext_attach_sigint(int32_t fd) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "event_source_fd", json_mknumber(fd));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    const size_t output_len = 16;
    char output[output_len];

    int err =
        __syscall("attach_sigint", serialized, (uint8_t *)output, output_len);
    free(serialized);
    return -err;
}
#endif

int wasi_ext_clean_inodes() {
    const size_t output_len = 4;
    char output[output_len];
    return __syscall("clean_inodes", "{}", (uint8_t *)output, output_len);
}

int wasi_ext_spawn(const char *path, const char *const *args, size_t n_args,
                   const struct Env *env, size_t n_env, int background,
                   const struct Redirect *redirects, size_t n_redirects) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "path", json_mkstring(path));

    JsonNode *_args = json_mkarray();
    for (size_t i = 0; i < n_args; i++) {
        json_append_element(_args, json_mkstring(args[i]));
    }
    json_append_member(root, "args", _args);

    JsonNode *_env = json_mkobject();
    for (size_t i = 0; i < n_env; i++) {
        json_append_member(_env, env[i].attrib, json_mkstring(env[i].val));
    }
    json_append_member(root, "extended_env", _env);

    json_append_member(root, "background", json_mkbool((bool)background));

    JsonNode *_redirects = json_mkarray();
    for (size_t i = 0; i < n_redirects; i++) {
        json_append_element(_redirects, json_mkredirect(redirects[i]));
    }
    json_append_member(root, "redirects", _redirects);

    char *call_args = json_stringify(0, root, " ");
    json_delete(root);

    const size_t output_len = 4;
    char buf[output_len];
    int result = __syscall("spawn", call_args, (uint8_t *)buf, output_len);
    free(call_args);
    int status = atoi(buf);
    if (status != 0)
        return -status;
    else
        return result;
}

int wasi_ext_kill(int pid, int sig) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "process_id", json_mknumber(pid));
    json_append_member(root, "signal", json_mknumber(sig));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("kill", serialized, NULL, 0);
    free(serialized);

    return -err;
}