#ifndef c_bindings_wasi_ext_lib_h_INCLUDED
#define c_bindings_wasi_ext_lib_h_INCLUDED

#include <stdlib.h>

int wasi_ext_chdir(const char*);
int wasi_ext_getcwd(char*, size_t);
int wasi_ext_isatty(int);
int wasi_ext_set_env(const char*, const char*);

#endif 

