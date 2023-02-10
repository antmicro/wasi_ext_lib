# Wasi Extended Library

Copyright (c) 2022-2023 [Antmicro](https://www.antmicro.com)

This repository contains a custom API for syscalls that are not yet implemented in `wasi`.
In order to keep compiled binaries compatible with other runtimes, these syscalls are not explicitly added to the [WASI standard](https://wasi.dev/).
Instead, they are redirected from the `path_readlink` function.

## Build

To build Rust and C libs you need [wasi-sdk](https://github.com/WebAssembly/wasi-sdk).
For installation instructions, refer to the official [WASI SDK installation guide](https://github.com/WebAssembly/wasi-sdk#install).
After completing the installation steps, set the `WASI_SDK_PATH` environment variable:
```
export WASI_SDK_PATH="/path/to/wasi-sdk"
```

### Rust library

You will need a custom Rust nightly toolchain.
The `canonicalize.patch` file should be applied to the `beta` branch of the official Rust repository.
Build the toolchain and `wasi_ext_lib` by following the steps below:

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

In order to build the C library, run:
```
(cd c_lib && make)
```

This command produces `libwasi_ext_lib.a` file in `c_lib/bin` directory.
It is a static library, you can link it by appending `-L<path_to_wasi_ext_lib>/c_lib/bin -lwasi_ext_lib` to the `clang` command.
