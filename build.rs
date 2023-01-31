// SPDX-License-Identifier: Apache-2.0
extern crate bindgen;

use std::env;
use std::process::Command;

const CLIB_DIR: &str = "c_lib";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={CLIB_DIR}/wasi_ext_lib.c");
    println!("cargo:rerun-if-changed={CLIB_DIR}/wasi_ext_lib.h");
    println!("cargo:rerun-if-changed={CLIB_DIR}/Makefile");
    let mut make = Command::new("make");
    #[cfg(feature = "hterm")]
    make.arg("CFLAGS=-DHTERM");
    if make
        .arg("-C")
        .arg(CLIB_DIR)
        .status()
        .expect("Could not build C library")
        .code()
        .unwrap()
        != 0
    {
        panic!("Unable to compile C library");
    };
    println!("cargo:rustc-link-search={CLIB_DIR}/bin/");

    println!("cargo:rustc-link-lib=static=wasi_ext_lib");
    println!("cargo:rerun-if-changed={CLIB_DIR}/wasi_ext_lib.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={CLIB_DIR}/wasi_ext_lib.c");
    println!("cargo:rerun-if-changed=src/lib.rs");
    bindgen::Builder::default()
        .header(format!("{CLIB_DIR}/wasi_ext_lib.h"))
        .clang_arg(format!(
            "--sysroot={}/share/wasi-sysroot",
            env!("WASI_SDK_PATH")
        ))
        .clang_arg("-DHTERM")
        .clang_arg("-fvisibility=default")
        .allowlist_file(format!("{CLIB_DIR}/wasi_ext_lib.h"))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/wasi_ext_lib_generated.rs")
        .expect("could not write bindings");
}
