# WASI_EXT_LIB_C

This repo contains `C` bindings for Antmicro's `wasi_ext_lib`. To build the library use:

```
CC="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SDK_PATH}/share/wasi-sysroot" cargo +stage2 build --target wasm32-wasi --release
```

This requires `wasi-sdk` and Antmicro's custom `rust` toolchain. After compiling, a static `C` library can be found in `target/wasm32-wasi/release`.

To generate headers use:
```

cbindgen --config cbindgen.toml
```

The output of this command is to be redirected to a desired path.
