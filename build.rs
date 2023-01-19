// SPDX-License-Identifier: Apache-2.0
extern crate bindgen;

use std::env;
use std::process::Command;

const CLIB_DIR: &str = "c_lib";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}/wasi_ext_lib.c", CLIB_DIR);
    println!("cargo:rerun-if-changed={}/wasi_ext_lib.h", CLIB_DIR);
    println!("cargo:rerun-if-changed={}/Makefile", CLIB_DIR);
    let mut make = Command::new("make");
    #[cfg(feature = "hterm")]
    make.arg("CFLAGS=-DHTERM");
    if make.arg("-C").arg(CLIB_DIR)
        .status()
        .expect("Could not build C library")
        .code().unwrap() != 0 {
        panic!("Unable to compile C library");
    };
    println!("cargo:rustc-link-search={}/bin/", CLIB_DIR);

    println!("cargo:rustc-link-lib=static=wasi_ext_lib");
    println!("cargo:rerun-if-changed={}/wasi_ext_lib.h", CLIB_DIR);
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}/wasi_ext_lib.c", CLIB_DIR);
    println!("cargo:rerun-if-changed=src/lib.rs");
    bindgen::Builder::default()
        .header(format!("{}/wasi_ext_lib.h", CLIB_DIR))
        .clang_arg(format!("--sysroot={}/share/wasi-sysroot", env!("WASI_SDK_PATH")))
        .clang_arg("-DHTERM")
        .clang_arg("-fvisibility=default")
        .allowlist_file(format!("{}/wasi_ext_lib.h", CLIB_DIR))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/wasi_ext_lib_generated.rs")
        .expect("could not write bindings");
}
