#ifndef c_bindings_wasi_ext_lib_h_INCLUDED
#define c_bindings_wasi_ext_lib_h_INCLUDED

#include <wasi/api.h>
#include <stdlib.h>

int wasi_ext_chdir(const char*);
int wasi_ext_getcwd(const char*, size_t);
int wasi_ext_isatty(__wasi_fd_t);
int wasi_ext_set_env(char*, char*);

#endif 

