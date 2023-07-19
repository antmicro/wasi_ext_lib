/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#ifndef c_bindings_wasi_ext_lib_h_INCLUDED
#define c_bindings_wasi_ext_lib_h_INCLUDED

#define _IOC_NONE 0U
#define _IOC_WRITE 1U
#define _IOC_READ 2U

#define _IOC(rw, maj, func, size) (rw << 30 | size << 16 | maj << 8 | func)
#define _IO(maj, func) _IOC(_IOC_NONE, maj, func, 0)
#define _IOW(maj, func, size) _IOC(_IOC_WRITE, maj, func, size)
#define _IOR(maj, func, size) _IOC(_IOC_READ, maj, func, size)
#define _IOWR(maj, func, size) _IOC(_IOC_WRITE | _IOC_READ, maj, func, size)

#include <stdlib.h>

// Ioctl magic numbers
const unsigned int TIOCGWINSZ = _IOR(1, 0, 8);
const unsigned int TIOCSRAW = _IOW(1, 1, 4);
const unsigned int TIOCSECHO = _IOW(1, 2, 4);

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
int wasi_ext_event_source_fd(uint32_t);
int wasi_ext_attach_sigint(int32_t);
#endif
int wasi_ext_clean_inodes();
int wasi_ext_spawn(const char *, const char *const *, size_t,
                   const struct Env *, size_t, int, const struct Redirect *,
                   size_t n_redirects, int *);
int wasi_ext_kill(int, int);
int wasi_ext_ioctl(int, unsigned int, void *);

#endif
