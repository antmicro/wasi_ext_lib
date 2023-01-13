# Wasi Extended Library

Copyright (c) 2022-2023 [Antmicro](https://www.antmicro.com).

This repo contains custom api for syscalls that are not yet implemented in `wasi`. In order to keep compiled binaries compatible with other runtimes, these syscalls are not explicitly added to wasi standard, but are redirected from `path_readlink`.

## Build

### Rust library

To build `wasi_ext_lib` there is needed custom nightly rust toolchain. `canonicalize.patch` file should be applied to official rust repository `beta` branch. Build toolchain as following following:

```
cargo build --target wasm32-wasi --release
```

### C library

```
(cd c_lib && make)
```
