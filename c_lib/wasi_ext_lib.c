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

int wasi_ext_mount(int source_fd, const char *source_path, int target_fd,
                   const char *target_path, const char *filesystem_type,
                   uint64_t mount_flags, const char *data) {
    JsonNode *root = json_mkobject();

    json_append_member(root, "source_fd", json_mknumber((double)source_fd));
    json_append_member(root, "source",
                       json_mknumber((double)(size_t)source_path));
    json_append_member(root, "source_len",
                       json_mknumber((double)strlen(source_path)));

    json_append_member(root, "target_fd", json_mknumber((double)target_fd));
    json_append_member(root, "target",
                       json_mknumber((double)(size_t)target_path));
    json_append_member(root, "target_len",
                       json_mknumber((double)strlen(target_path)));

    json_append_member(root, "filesystemtype",
                       json_mknumber((double)(size_t)filesystem_type));
    json_append_member(root, "filesystemtype_len",
                       json_mknumber((double)strlen(filesystem_type)));

    json_append_member(root, "mountflags", json_mknumber((double)mount_flags));

    json_append_member(root, "data", json_mknumber((double)(size_t)data));
    json_append_member(root, "data_len", json_mknumber((double)strlen(data)));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("mount", serialized, NULL, 0);
    free(serialized);

    return err;
}

int wasi_ext_umount(const char *path) {
    JsonNode *root = json_mkobject();

    json_append_member(root, "path", json_mknumber((double)(size_t)path));
    json_append_member(root, "path_len", json_mknumber(strlen(path)));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("umount", serialized, NULL, 0);
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
    int res = *((int *)output);
    return res;
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
        int res = *((int *)output);
        return res;
    }
}

#ifdef HTERM
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
    int res = *((int *)output);
    return res;
}

int wasi_ext_attach_sigint(int32_t fd) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "event_source_fd", json_mknumber(fd));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("attach_sigint", serialized, NULL, 0);
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
                   const struct Redirect *redirects, size_t n_redirects,
                   int *child_pid) {
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

    json_append_member(root, "redirects_ptr",
                       json_mknumber((double)((size_t)redirects)));

    json_append_member(root, "n_redirects", json_mknumber((double)n_redirects));

    char *call_args = json_stringify(0, root, " ");
    json_delete(root);

    const size_t output_len = 8;
    char buf[output_len];
    int result = __syscall("spawn", call_args, (uint8_t *)buf, output_len);
    free(call_args);
    int *data_ptr = (int *)buf;
    int status = data_ptr[0];
    *child_pid = data_ptr[1];
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

int wasi_ext_ioctl(int fd, unsigned int cmd, void *arg) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "fd", json_mknumber(fd));
    json_append_member(root, "cmd", json_mknumber(cmd));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("ioctl", serialized, arg, _IOGS(cmd));

    free(serialized);

    return -err;
}

int wasi_ext_fcntl(int fd, enum FcntlCommand cmd, void *arg) {
    __wasi_errno_t err;
    switch (cmd) {
    case F_MVFD: {
        int min_fd = *((int *)arg);
        __wasi_fdstat_t stat;

        for (; min_fd < _MAX_FD_NUM; ++min_fd) {
            err = __wasi_fd_fdstat_get(min_fd, &stat);

            if (__WASI_ERRNO_BADF == err) {
                break;
            } else if (__WASI_ERRNO_SUCCESS != err) {
                return -err;
            }
        }

        if (min_fd >= _MAX_FD_NUM) {
            return __WASI_ERRNO_MFILE;
        }

        // We assume fd_renumber behaves like dup2
        err = __wasi_fd_renumber(fd, min_fd);
        if (__WASI_ERRNO_SUCCESS != err) {
            return -err;
        }

        err = __wasi_fd_close(fd);
        if (__WASI_ERRNO_SUCCESS != err) {
            return -err;
        }

        // like F_DUPFD, return allocated fd
        return min_fd;
    }
    case F_GETFD: {
        __wasi_fdstat_t stat;
        err = __wasi_fd_fdstat_get(fd, &stat);

        if (__WASI_ERRNO_SUCCESS != err) {
            return -err;
        }

        __wasi_fdflags_t flags = stat.fs_flags & WASI_EXT_FDFLAG_MASK;

        return (int)flags;
    }
    case F_SETFD: {
        __wasi_fdflags_t flags = *((__wasi_fdflags_t *)arg);
        // set control bit to enable extended flags processing
        flags |= WASI_EXT_FDFLAG_CTRL_BIT;

        err = __wasi_fd_fdstat_set_flags(fd, flags);

        return -err;
    }
    }

    return -EINVAL;
}

int wasi_ext_mknod(const char *path, int dev) {
    JsonNode *root = json_mkobject();
    json_append_member(root, "path", json_mkstring(path));
    json_append_member(root, "dev", json_mknumber(dev));

    char *serialized = json_stringify(0, root, " ");
    json_delete(root);

    int err = __syscall("mknod", serialized, NULL, 0);
    free(serialized);

    return err;
}
