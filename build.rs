// SPDX-License-Identifier: Apache-2.0
extern crate bindgen;

use std::env;
use std::process::Command;

const CLIB_DIR: &str = "c_lib";
const CLIB_THIRD_PARTY_DIR: &str = "c_lib/third_party";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={CLIB_THIRD_PARTY_DIR}/termios/termios.c");
    println!("cargo:rerun-if-changed={CLIB_THIRD_PARTY_DIR}/termios/termios.h");
    println!("cargo:rerun-if-changed={CLIB_THIRD_PARTY_DIR}/termios/*");
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
    println!("cargo:rerun-if-changed={CLIB_THIRD_PARTY_DIR}/termios/*");
    println!("cargo:rerun-if-changed={CLIB_THIRD_PARTY_DIR}/termios/termios.h");
    println!("cargo:rerun-if-changed={CLIB_THIRD_PARTY_DIR}/termios/termios.c");
    println!("cargo:rerun-if-changed={CLIB_DIR}/wasi_ext_lib.h");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={CLIB_DIR}/wasi_ext_lib.c");
    println!("cargo:rerun-if-changed=src/lib.rs");
    let mut bgen = bindgen::Builder::default()
        .header(format!("{CLIB_DIR}/wasi_ext_lib.h"));
    if cfg!(feature = "hterm") {
        bgen = bgen.clang_arg("-DHTERM");
    }
    bgen.clang_arg(format!(
        "--sysroot={}/share/wasi-sysroot",
        env!("WASI_SDK_PATH")
    ))
    .clang_arg("-fvisibility=default")
    .allowlist_file(format!("{CLIB_DIR}/wasi_ext_lib.h"))
    .allowlist_file(format!("{CLIB_THIRD_PARTY_DIR}/termios/termios.h"))
    .allowlist_file(format!("{CLIB_THIRD_PARTY_DIR}/termios/bits/termios.h"))
    .parse_callbacks(Box::new(bindgen::CargoCallbacks))
    .generate()
    .expect("Unable to generate bindings")
    .write_to_file("src/wasi_ext_lib_generated.rs")
    .expect("could not write bindings");
}
