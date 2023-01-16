# Wasi Extended Library

Copyright (c) 2022-2023 [Antmicro](https://www.antmicro.com).

This repo contains custom api for syscalls that are not yet implemented in `wasi`. In order to keep compiled binaries compatible with other runtimes, these syscalls are not explicitly added to wasi standard, but are redirected from `path_readlink`.

## Build

### Rust library
There is needed custom nightly Rust toolchain. `canonicalize.patch` file should be applied to `beta` branch of the official Rust repository. Build toolchain and `wasi_ext_lib` as following:

```
# clone repos
git clone https://github.com/antmicro/wasi_ext_lib.git
git clone https://github.com/rust-lang/rust.git

# apply patch to official Rust repo
cd rust
git checkout beta
git apply ../wasi_ext_lib/canonicalize.patch

# build toolchain, remember to meet all dependencies required by Rust
mkdir tmp
cd tmp
../src/ci/docker/host-x86_64/dist-various-2/build-wasi-toolchain.sh
cd ..
./configure --target=wasm32-wasi --disable-docs --set target.wasm32-wasi.wasi-root=/wasm32-wasi --enable-lld --tools=cargo
./x.py build --target x86_64-unknown-linux-gnu --target wasm32-wasi --stage 2

# link toolchain and build `wasi_ext_lib`
rustup toolchain link stage2 "$(pwd)/build/x86_64-unknown-linux-gnu/stage2"
cd ../wasi_ext_lib
cargo +stage2 build --target wasm32-wasi --release
```

### C library

```
(cd c_lib && make)
```
