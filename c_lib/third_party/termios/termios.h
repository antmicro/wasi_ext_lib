/* SPDX-License-Identifier: MIT */

// https://github.com/WebAssembly/wasi-libc/blob/main/libc-top-half/musl/include/termios.h
// https://github.com/WebAssembly/wasi-libc/blob/main/libc-top-half/musl/arch/generic/bits/termios.h

#ifndef	_TERMIOS_H
#define	_TERMIOS_H

#include <features.h>

#define __NEED_pid_t
#define __NEED_struct_winsize

#include <bits/alltypes.h>


typedef unsigned char cc_t;
typedef unsigned int speed_t;
typedef unsigned int tcflag_t;

#define NCCS 32

#include "bits/termios.h"
#include "../../wasi_ext_lib.h"

// Ioctl magic numbers
#define TCGETS _IOR(1, 1, 0)
#define TCSETS _IOW(1, 2, 0)

speed_t wasi_ext_cfgetospeed (const struct termios *);
speed_t wasi_ext_cfgetispeed (const struct termios *);
int wasi_ext_cfsetospeed (struct termios *, speed_t);
int wasi_ext_cfsetispeed (struct termios *, speed_t);

int wasi_ext_tcgetattr (int, struct termios *);
int wasi_ext_tcsetattr (int, int, const struct termios *);

int wasi_ext_tcgetwinsize (int, struct winsize *);
int wasi_ext_tcsetwinsize (int, const struct winsize *);

int wasi_ext_tcsendbreak (int, int);
int wasi_ext_tcdrain (int);
int wasi_ext_tcflush (int, int);
int wasi_ext_tcflow (int, int);

pid_t wasi_ext_tcgetsid (int);

void wasi_ext_cfmakeraw(struct termios *);
int wasi_ext_cfsetspeed(struct termios *, speed_t);

#endif
