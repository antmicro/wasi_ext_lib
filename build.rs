extern crate bindgen;
use std::env;

fn main() {
    cc::Build::new()
        .archiver(format!("{}/bin/ar", env!("WASI_SDK_PATH")))
        .file("c_bindings/wasi_ext_lib.c")
        .flag("-DHTERM")
        .compile("wasi_ext_lib");

    println!("cargo:rustc-link-lib=static=wasi_ext_lib");
    println!("cargo:rerun-if-changed=c_bindings/wasi_ext_lib.h");
    println!("cargo:rerun-if-changed=c_bindings/wasi_ext_lib.c");
    bindgen::Builder::default()
        .header("c_bindings/wasi_ext_lib.h")
        .clang_arg(format!("--sysroot={}/share/wasi-sysroot", env!("WASI_SDK_PATH")))
        .clang_arg("-fvisibility=default")
        .allowlist_file("c_bindings/wasi_ext_lib.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/wasi_ext_lib_generated.rs")
        .expect("could not write bindings");
}
