// Copyright 2019 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE-ChromiumOS file.

// https://pubs.opengroup.org/onlinepubs/9799919799/basedefs/sys_un.h.html

#ifndef WASI_C_SUP_SYS_UN_H
#define WASI_C_SUP_SYS_UN_H

#include <sys/socket.h>

struct sockaddr_un {
  sa_family_t sun_family;
  char sun_path[108];
};

#endif
