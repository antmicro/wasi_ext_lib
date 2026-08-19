#ifndef WASI_C_SUP_H
#define WASI_C_SUP_H

#include <sys/cdefs.h>

__BEGIN_DECLS

void wasi_ext_signal_deliver(int signum);

__END_DECLS

#endif
