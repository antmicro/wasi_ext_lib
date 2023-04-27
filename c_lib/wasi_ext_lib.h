/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef c_bindings_wasi_ext_lib_h_INCLUDED
#define c_bindings_wasi_ext_lib_h_INCLUDED

#include <stdlib.h>

enum RedirectType { READ, WRITE, APPEND };

struct Redirect {
    int fd;
    const char *path;
    enum RedirectType type;
};
struct Env {
    const char *attrib;
    const char *val;
};

#ifdef HTERM
typedef uint32_t WasiEvents;
const size_t WASI_EVENTS_NUM = 2;
const size_t WASI_EVENTS_MASK_SIZE = 4; // number of bytes
// Hterm events
const WasiEvents WASI_EVENT_WINCH = 1 << 0;
const WasiEvents WASI_EVENT_SIGINT = 1 << 1;
#endif

int wasi_ext_chdir(const char *);
int wasi_ext_getcwd(char *, size_t);
int wasi_ext_isatty(int);
int wasi_ext_set_env(const char *, const char *);
int wasi_ext_getpid();
int wasi_ext_set_echo(int);
#ifdef HTERM
int wasi_ext_hterm_get(const char *, char *, size_t);
int wasi_ext_hterm_set(const char *, const char *);
int wasi_ext_event_source_fd(uint32_t);
int wasi_ext_attach_sigint(int32_t);
#endif
int wasi_ext_clean_inodes();
int wasi_ext_spawn(const char *, const char *const *, size_t,
                   const struct Env *, size_t, int, const struct Redirect *,
                   size_t n_redirects);
int wasi_ext_kill(int, int);

#endif
