// Copyright 2019 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-ChromiumOS file.

// https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_select.h.html

#ifndef WASI_C_SUP_SYS_SELECT_H
#define WASI_C_SUP_SYS_SELECT_H

#include_next <sys/select.h>

typedef unsigned long fd_mask;

#endif
