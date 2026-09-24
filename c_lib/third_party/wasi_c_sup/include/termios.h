// Copyright 2019 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-ChromiumOS file.

// https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/termios.h.html

#ifndef WASI_C_SUP_TERMIOS_H
#define WASI_C_SUP_TERMIOS_H

#include <sys/cdefs.h>
#include <sys/types.h>
#include <third_party/termios/termios.h>

__BEGIN_DECLS

speed_t cfgetispeed(const struct termios*);
speed_t cfgetospeed(const struct termios*);
int cfsetispeed(struct termios*, speed_t);
int cfsetospeed(struct termios*, speed_t);
int tcdrain(int);
int tcflow(int, int);
int tcflush(int, int);
int tcgetattr(int, struct termios*);
int tcsendbreak(int, int);
int tcsetattr(int, int, const struct termios*);

__END_DECLS

#endif
