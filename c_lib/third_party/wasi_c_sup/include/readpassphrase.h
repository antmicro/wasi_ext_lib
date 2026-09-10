// Copyright 2020 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-ChromiumOS file.

// https://linux.die.net/man/3/readpassphrase

#ifndef WASI_C_SUP_READPASSPHRASE_H
#define WASI_C_SUP_READPASSPHRASE_H

#include <stddef.h>
#include <sys/cdefs.h>

#define RPP_ECHO_OFF    0x00
#define RPP_ECHO_ON     0x01
#define RPP_REQUIRE_TTY 0x02
#define RPP_FORCELOWER  0x04
#define RPP_FORCEUPPER  0x08
#define RPP_SEVENBIT    0x10
#define RPP_STDIN       0x20

__BEGIN_DECLS

char* readpassphrase(const char* prompt, char* buf, size_t buf_len, int flags);

__END_DECLS

#endif
