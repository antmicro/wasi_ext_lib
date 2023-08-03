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

#define _IORW_OFF 30
#define _IOS_OFF 16
#define _IOM_OFF 8
#define _IOF_OFF 0

#define _IORW_MASK 0xc0000000
#define _IOS_MASK 0x3fff0000
#define _IOM_MASK 0x0000ff00
#define _IOF_MASK 0x000000ff

#define _IOC(rw, maj, func, size)                                              \
    (rw << _IORW_OFF | size << _IOS_OFF | maj << _IOM_OFF | func << _IOF_OFF)

#define _IO(maj, func) _IOC(_IOC_NONE, maj, func, 0)
#define _IOW(maj, func, size) _IOC(_IOC_WRITE, maj, func, size)
#define _IOR(maj, func, size) _IOC(_IOC_READ, maj, func, size)
#define _IOWR(maj, func, size) _IOC(_IOC_WRITE | _IOC_READ, maj, func, size)

#define _IOGRW(mn) ((mn & _IORW_MASK) >> _IORW_OFF)
#define _IOGS(mn) ((mn & _IOS_MASK) >> _IOS_OFF)
#define _IOGM(mn) ((mn & _IOM_MASK) >> _IOM_OFF)
#define _IOGF(mn) ((mn & _IOF_MASK) >> _IOF_OFF)

#include <stdlib.h>

// Ioctl magic numbers
const unsigned int TIOCGWINSZ = _IOR(1, 0, 8);
const unsigned int TIOCSRAW = _IOW(1, 1, 4);
const unsigned int TIOCSECHO = _IOW(1, 2, 4);

// Fnctl commands
enum FcntlCommand { F_MVFD };

const int STDIN = 0;
const int STDOUT = 1;

enum RedirectType {
    READ,
    WRITE,
    APPEND,
    READWRITE,
    PIPEIN,
    PIPEOUT,
    DUPLICATE,
    CLOSE
};

struct Redirect {
    union Data {
        struct Path {
            const char *path_str;
            size_t path_len;
        } path;

        int fd_src;
    } data;

    int fd_dst;
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
                   size_t, int *);
int wasi_ext_kill(int, int);
int wasi_ext_ioctl(int, unsigned int, void *);
int wasi_ext_fcntl(int, enum FcntlCommand, void *);

#endif
