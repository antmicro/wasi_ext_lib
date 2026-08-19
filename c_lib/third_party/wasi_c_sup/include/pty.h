// Copyright 2025 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

// https://man7.org/linux/man-pages/man3/openpty.3.html

#ifndef WASI_C_SUP_PTY_H
#define WASI_C_SUP_PTY_H

#include <sys/cdefs.h>
#include <sys/types.h>
#include <sys/ioctl.h>
#include <termios.h>

__BEGIN_DECLS

int openpty(int* amaster,
            int* aslave,
            char* name,
            const struct termios* termp,
            const struct winsize* winp);

__END_DECLS

#endif
