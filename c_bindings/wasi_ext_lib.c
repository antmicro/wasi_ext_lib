#include <wasi/api.h>

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <stdbool.h>

#include "wasi_ext_lib.h"
#include "json/json.h"

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

int __syscall(const char *command, char *args, uint8_t *output_buf, size_t output_buf_len) {
    char c[SYSCALL_LENGTH];
    sprintf(c, "!{\"command\": \"%s\", \"buf_len\": %ld, \"buf_ptr\": \"%p\"}", command, strlen(args), args);

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
    sprintf(c, "{\"buf_len\": %zu}", buf_len);
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
    char *env = malloc(strlen(attrib) + strlen(val) + 2);
    sprintf(env, "%s=%s", attrib, val);
    free(env);
    if (val != NULL) {
        sprintf(args, "{ \"attrib\": %s, \"val\": %s }", attrib, val);
    } else {
        sprintf(args, "{ \"attrib\": %s }", attrib);
    }
    return __syscall("set_env", args, NULL, 0);
}

int wasi_ext_getpid() {
    char args[] = "{}";
    const size_t output_len = 16;
    char output[output_len];
    int result = __syscall("getpid", args, (uint8_t*)output, output_len);
    if (result != 0) {
        return -result;
    } else {
        return atoi(output);
    }
}

int wasi_ext_set_echo(int should_echo) {
    char args[SYSCALL_ARGS_LENGTH];
    sprintf(args, "{ \"echo\": %d }", should_echo);
    return __syscall("set_echo", args, NULL, 0);
}

#ifdef HTERM
int wasi_ext_hterm_set(const char* attrib, const char *val) {
    char args[SYSCALL_ARGS_LENGTH];
    sprintf(args, "{ \"method\": \"set\", \"attrib\": %s, \"val\": %s }", attrib, val);
    return __syscall("hterm", args, NULL, 0);
}

int wasi_ext_hterm_get(const char* attrib, char *val, size_t val_len) {
    char args[SYSCALL_ARGS_LENGTH];
    sprintf(args, "{ \"method\": \"get\", \"attrib\": %s }", attrib);
    return __syscall("hterm", args, (uint8_t*)val, val_len);
}
#endif

int wasi_ext_spawn(
    const char *path,
    const char *const *args,
    size_t n_args,
    const struct Env *env,
    size_t n_env,
    int background,
    const struct Redirect *redirects,
    size_t n_redirects
) {
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
    json_append_member(root, "env", _env);

    json_append_member(root, "background", json_mkbool((bool)background));

    JsonNode *_redirects = json_mkarray();
    for (size_t i = 0; i < n_redirects; i++) {
        json_append_element(_redirects, json_mkredirect(redirects[i]));
    }
    json_append_member(root, "redirects", _redirects);

    char call_args[SYSCALL_ARGS_LENGTH];
    json_stringify(root, call_args);
    json_delete(root);

    const size_t output_len = 4;
    char buf[output_len];
    int result = __syscall("spawn", call_args, buf, output_len);
    int status = atoi(buf);
    if (status != 0) return -status;
    else return result;
}
