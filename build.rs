extern crate bindgen;
use std::env;

fn main() {
    cc::Build::new()
        .archiver(format!("{}/bin/ar", env!("WASI_SDK_PATH")))
        .file("c_bindings/wasi_ext_lib.c")
        .compile("wasi_ext_lib");

    println!("cargo:rustc-link-lib=static=wasi_ext_lib");
    println!("cargo:rerun-if-changed=c_bindings/wasi_ext_lib.h");
    /* bindgen::Builder::default()
        .header("c_bindings/wasi_ext_lib.h")
        // .clang_arg(format!("{}/share/wasi-sysroot/include", env!("WASI_SDK_PATH")))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/bindings.rs")
        .expect("could not write bindings"); */
}
