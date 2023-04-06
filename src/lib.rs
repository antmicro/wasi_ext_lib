/*
 * Copyright (c) 2022-2023 Antmicro <www.antmicro.com>
 *
 * SPDX-License-Identifier: Apache-2.0
 */
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::collections::HashMap;
use std::convert::AsRef;
use std::convert::From;
use std::env;
use std::ffi::CString;
use std::fs;
use std::os::wasi::ffi::OsStrExt;
#[cfg(feature = "hterm")]
use std::os::wasi::prelude::RawFd;
use std::path::Path;
use std::ptr;
use std::str;

mod wasi_ext_lib_generated;
#[cfg(feature = "hterm")]
pub use wasi_ext_lib_generated::{
    WasiEvents, WASI_EVENTS_MASK_SIZE, WASI_EVENTS_NUM, WASI_EVENT_WINCH, WASI_EVENT_SIGINT
};

type ExitCode = i32;
type Pid = i32;

pub enum Redirect<'a> {
    Read((wasi::Fd, &'a str)),
    Write((wasi::Fd, &'a str)),
    Append((wasi::Fd, &'a str)),
}

enum CStringRedirect {
    Read((wasi::Fd, CString)),
    Write((wasi::Fd, CString)),
    Append((wasi::Fd, CString)),
}

impl From<Redirect<'_>> for CStringRedirect {
    fn from(redirect: Redirect) -> Self {
        match redirect {
            Redirect::Read((fd, path)) => CStringRedirect::Read((fd, CString::new(path).unwrap())),
            Redirect::Write((fd, path)) => {
                CStringRedirect::Write((fd, CString::new(path).unwrap()))
            }
            Redirect::Append((fd, path)) => {
                CStringRedirect::Append((fd, CString::new(path).unwrap()))
            }
        }
    }
}

unsafe fn get_c_redirect(r: &CStringRedirect) -> wasi_ext_lib_generated::Redirect {
    match r {
        CStringRedirect::Read((fd, path)) => wasi_ext_lib_generated::Redirect {
            type_: wasi_ext_lib_generated::RedirectType_READ,
            path: path.as_c_str().as_ptr(),
            fd: *fd as i32,
        },
        CStringRedirect::Write((fd, path)) => wasi_ext_lib_generated::Redirect {
            type_: wasi_ext_lib_generated::RedirectType_WRITE,
            path: path.as_c_str().as_ptr(),
            fd: *fd as i32,
        },
        CStringRedirect::Append((fd, path)) => wasi_ext_lib_generated::Redirect {
            type_: wasi_ext_lib_generated::RedirectType_APPEND,
            path: path.as_c_str().as_ptr(),
            fd: *fd as i32,
        },
    }
}

pub fn chdir<P: AsRef<Path>>(path: P) -> Result<(), ExitCode> {
    if let Ok(canon) = fs::canonicalize(path.as_ref()) {
        if let Err(e) = env::set_current_dir(canon.as_path()) {
            return Err(e
                .raw_os_error()
                .unwrap_or_else(|| wasi::ERRNO_INVAL.raw().into()));
        };
        let pth = match CString::new(canon.as_os_str().as_bytes()) {
            Ok(p) => p,
            Err(_) => return Err(wasi::ERRNO_INVAL.raw().into()),
        };
        match unsafe { wasi_ext_lib_generated::wasi_ext_chdir(pth.as_ptr()) } {
            0 => Ok(()),
            e => Err(e),
        }
    } else {
        Err(wasi::ERRNO_INVAL.raw().into())
    }
}

pub fn getcwd() -> Result<String, ExitCode> {
    const MAX_BUF_SIZE: usize = 65536;
    let mut buf_size: usize = 256;
    let mut buf = vec![0u8; buf_size];
    while buf_size < MAX_BUF_SIZE {
        match unsafe {
            wasi_ext_lib_generated::wasi_ext_getcwd(buf.as_mut_ptr() as *mut i8, buf_size)
        } {
            0 => {
                return Ok(String::from(
                    str::from_utf8(&buf[..buf.iter().position(|&i| i == 0).unwrap()]).unwrap(),
                ))
            }
            e => {
                if e != wasi::ERRNO_NOBUFS.raw().into() {
                    return Err(e);
                };
            }
        };
        buf_size *= 2;
        buf.resize(buf_size, 0u8);
    }
    Err(wasi::ERRNO_NAMETOOLONG.raw().into())
}

pub fn isatty(fd: i32) -> Result<bool, ExitCode> {
    let result = unsafe { wasi_ext_lib_generated::wasi_ext_isatty(fd) };
    if result < 0 {
        Err(-result)
    } else {
        Ok(result == 1)
    }
}

pub fn set_env(key: &str, val: Option<&str>) -> Result<(), ExitCode> {
    let c_key = CString::new(key).unwrap();
    match if let Some(v) = val {
        let c_val = CString::new(v).unwrap();
        unsafe {
            wasi_ext_lib_generated::wasi_ext_set_env(
                c_key.as_ptr() as *const i8,
                c_val.as_ptr() as *const i8,
            )
        }
    } else {
        unsafe {
            wasi_ext_lib_generated::wasi_ext_set_env(c_key.as_ptr() as *const i8, ptr::null::<i8>())
        }
    } {
        0 => Ok(()),
        e => Err(e),
    }
}

pub fn getpid() -> Result<Pid, ExitCode> {
    let result = unsafe { wasi_ext_lib_generated::wasi_ext_getpid() };
    if result < 0 {
        Err(-result)
    } else {
        Ok(result)
    }
}

pub fn set_echo(should_echo: bool) -> Result<(), ExitCode> {
    match unsafe { wasi_ext_lib_generated::wasi_ext_set_echo(should_echo as i32) } {
        0 => Ok(()),
        e => Err(e),
    }
}

#[cfg(feature = "hterm")]
pub fn hterm(attrib: &str, val: Option<&str>) -> Result<Option<String>, ExitCode> {
    match val {
        Some(value) => {
            match unsafe {
                wasi_ext_lib_generated::wasi_ext_hterm_set(
                    CString::new(attrib).unwrap().as_c_str().as_ptr() as *const i8,
                    CString::new(value).unwrap().as_c_str().as_ptr() as *const i8,
                )
            } {
                0 => Ok(None),
                e => Err(e),
            }
        }
        None => {
            const OUTPUT_LEN: usize = 256;
            let mut buf = [0u8; OUTPUT_LEN];
            match unsafe {
                wasi_ext_lib_generated::wasi_ext_hterm_get(
                    CString::new(attrib).unwrap().as_c_str().as_ptr() as *const i8,
                    buf.as_mut_ptr() as *mut i8,
                    OUTPUT_LEN,
                )
            } {
                0 => Ok(Some(
                    str::from_utf8(
                        &buf[..match buf.iter().position(|&i| i == 0) {
                            Some(x) => x,
                            None => {
                                return Err(wasi::ERRNO_ILSEQ.raw().into());
                            }
                        }],
                    )
                    .expect("Could not read syscall output")
                    .to_string(),
                )),
                e => Err(e),
            }
        }
    }
}

#[cfg(feature = "hterm")]
pub fn event_source_fd(event_mask: WasiEvents) -> Result<RawFd, ExitCode> {
    let result = unsafe { wasi_ext_lib_generated::wasi_ext_event_source_fd(event_mask) };
    if result < 0 {
        Err(-result)
    } else {
        Ok(result)
    }
}

#[cfg(feature = "hterm")]
pub fn attach_sigint(fd: RawFd) -> Result<(), ExitCode> {
    let result = unsafe { wasi_ext_lib_generated::wasi_ext_attach_sigint(fd) };
    if result < 0 {
        Err(-result)
    } else {
        Ok(())
    }
}

pub fn clean_inodes() -> Result<(), ExitCode> {
    match unsafe { wasi_ext_lib_generated::wasi_ext_clean_inodes() } {
        0 => Ok(()),
        n => Err(n),
    }
}

pub fn spawn(
    path: &str,
    args: &[&str],
    env: &HashMap<String, String>,
    background: bool,
    redirects: Vec<Redirect>,
) -> Result<ExitCode, ExitCode> {
    let syscall_result = unsafe {
        let cstring_args = args
            .iter()
            .map(|arg| CString::new(*arg).unwrap())
            .collect::<Vec<CString>>();

        let cstring_env = env
            .iter()
            .map(|(key, val)| {
                (
                    CString::new(&key[..]).unwrap(),
                    CString::new(&val[..]).unwrap(),
                )
            })
            .collect::<Vec<(CString, CString)>>();

        let cstring_redirects = redirects
            .into_iter()
            .map(CStringRedirect::from)
            .collect::<Vec<CStringRedirect>>();

        wasi_ext_lib_generated::wasi_ext_spawn(
            CString::new(path).unwrap().as_c_str().as_ptr(),
            cstring_args
                .iter()
                .map(|arg| arg.as_c_str().as_ptr())
                .collect::<Vec<*const i8>>()
                .as_ptr(),
            args.len(),
            cstring_env
                .iter()
                .map(|(key, val)| wasi_ext_lib_generated::Env {
                    attrib: key.as_c_str().as_ptr(),
                    val: val.as_c_str().as_ptr(),
                })
                .collect::<Vec<wasi_ext_lib_generated::Env>>()
                .as_ptr(),
            env.len(),
            background as i32,
            cstring_redirects
                .iter()
                .map(|red| get_c_redirect(red))
                .collect::<Vec<wasi_ext_lib_generated::Redirect>>()
                .as_ptr(),
            cstring_redirects.len(),
        )
    };
    if syscall_result < 0 {
        Err(-syscall_result)
    } else {
        Ok(syscall_result)
    }
}
