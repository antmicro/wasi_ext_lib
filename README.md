# Wasi Extended Library

This repo contains custom api for syscalls that are not yet implemented in `wasi`. In order to keep compiled binaries compatible with other runtimes, these syscalls are not explicitly added to wasi standard, but are redirected from `path_readlink`.

## Build

### Rust library

```
cargo build --target wasm32-wasi --release
```

### C library

```
(cd c_lib && make)
```
