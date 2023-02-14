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
Build the toolchain by following the steps below:

```
# clone repos
git clone https://github.com/antmicro/wasi_ext_lib.git
git clone https://github.com/rust-lang/rust.git

# apply patch to official Rust repo
cd rust
git checkout beta
git apply ../wasi_ext_lib/canonicalize.patch

# build toolchain, remember to meet all dependencies required by Rust
./configure --target=wasm32-wasi --disable-docs --set target.wasm32-wasi.wasi-root=${WASI_SDK_PATH}/share/wasi-sysroot --enable-lld --tools=cargo
./x.py build --target wasm32-wasi --target x86_64-unknown-linux-gnu --stage 1

# link toolchain and build `wasi_ext_lib`
rustup toolchain link wasi_extended "$(pwd)/build/host/stage1"
```

Note that `rustup toolchain link` command only creates a symlink to the given target.
If you choose to remove rust sources after building the toolchain, make sure that `stage1` directory is still under the linked path (the directory can be moved somewhere else and linked from there).

After the toolchain is installed, the library can be compiled using:

```
cargo +wasi_extended build --target wasm32-wasi --release
```

### C library

In order to build the C library, run:
```
(cd c_lib && make)
```

This command produces `libwasi_ext_lib.a` file in `c_lib/bin` directory.
It is a static library, you can link it by appending `-L<path_to_wasi_ext_lib>/c_lib/bin -lwasi_ext_lib` to the `clang` command.
