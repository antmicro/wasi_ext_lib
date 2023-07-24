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
use std::ffi::{c_uint, c_void, CString};
use std::fs;
use std::os::fd::RawFd;
use std::os::wasi::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::str;
use std::mem;

mod wasi_ext_lib_generated;

#[cfg(feature = "hterm")]
pub use wasi_ext_lib_generated::{
    WasiEvents, TIOCGWINSZ, TIOCSECHO, TIOCSRAW, WASI_EVENTS_MASK_SIZE, WASI_EVENTS_NUM,
    WASI_EVENT_SIGINT, WASI_EVENT_WINCH,
};

pub use wasi::SIGNAL_KILL;

type ExitCode = i32;
type Pid = i32;

pub enum Redirect<'a> {
    Read((wasi::Fd, &'a str)),
    Write((wasi::Fd, &'a str)),
    Append((wasi::Fd, &'a str)),
    ReadWrite((wasi::Fd, &'a str)),
    PipeIn(wasi::Fd),
    PipeOut(wasi::Fd),
    Duplicate { fd_src: wasi::Fd, fd_dst: wasi::Fd },
    Close(wasi::Fd),
}

#[repr(u32)]
pub enum IoctlNum {
    GetScreenSize = wasi_ext_lib_generated::TIOCGWINSZ,
    SetRaw = wasi_ext_lib_generated::TIOCSRAW,
    SetEcho = wasi_ext_lib_generated::TIOCSECHO,
}

const REDIREDT_TAG_READ: u8 = 0x1;
const REDIREDT_TAG_WRITE: u8 = 0x2;
const REDIREDT_TAG_APPEND: u8 = 0x3;
const REDIREDT_TAG_READWRITE: u8 = 0x4;
const REDIREDT_TAG_PIPEIN: u8 = 0x5;
const REDIREDT_TAG_PIPEOUT: u8 = 0x6;
const REDIREDT_TAG_DUPLICATE: u8 = 0x7;
const REDIREDT_TAG_CLOSE: u8 = 0x8;

type PathData = (*const u8, usize);
/*
Data type size and alignment, printed by: `cargo +nightly rustc --target wasm32-wasi -- -Zprint-type-sizes`
type: `RedirectDataU`: 8 bytes, alignment: 4 bytes
    variant `RedirectDataU`: 8 bytes
        field `.fd_src`: 4 bytes
        field `.path`: 8 bytes, offset: 0 bytes, alignment: 4 bytes
*/
union RedirectDataU {
    pub path: PathData,
    pub fd_src: u32,
}

/*
Data type size and alignment, printed by: `cargo +nightly rustc --target wasm32-wasi -- -Zprint-type-sizes`
type: `InternalRedirect`: 16 bytes, alignment: 4 bytes
    field `.data`: 8 bytes
    field `.fd`: 4 bytes
    field `.tag`: 1 bytes
    end padding: 3 bytes
*/
struct InternalRedirect {
    data: RedirectDataU,
    fd: wasi::Fd,
    tag: u8,
}

impl From<Redirect<'_>> for InternalRedirect {
    fn from(redirect: Redirect) -> Self {
        match redirect {
            Redirect::Read((fd, path)) |
            Redirect::Write((fd, path)) |
            Redirect::Append((fd, path)) |
            Redirect::ReadWrite((fd, path)) => {
                let tag = match redirect {
                    Redirect::Read((fd, path)) => REDIREDT_TAG_READ,
                    Redirect::Write((fd, path)) => REDIREDT_TAG_WRITE,
                    Redirect::Append((fd, path)) => REDIREDT_TAG_APPEND,
                    Redirect::ReadWrite((fd, path)) => REDIREDT_TAG_READWRITE,
                    _ => unreachable!()
                };

                InternalRedirect {
                    data: RedirectDataU {
                        path: (
                            path.as_ptr(),
                            path.len(),
                        )
                    },
                    fd,
                    tag,
                }
            }
            Redirect::PipeIn(fd_src) => InternalRedirect {
                data: RedirectDataU { fd_src },
                fd: 0, //TODO: pass it by constant
                tag: REDIREDT_TAG_PIPEIN,
            },
            Redirect::PipeOut(fd_src) => InternalRedirect {
                data: RedirectDataU { fd_src },
                fd: 1, //TODO: pass it by constant
                tag: REDIREDT_TAG_PIPEOUT,
            },
            Redirect::Duplicate { fd_src, fd_dst } => InternalRedirect {
                data: RedirectDataU { fd_src },
                fd: fd_dst,
                tag: REDIREDT_TAG_DUPLICATE,
            },
            Redirect::Close(fd_dst) => InternalRedirect {
                data: unsafe { mem::zeroed() }, // ignore field in kernel
                fd: fd_dst,
                tag: REDIREDT_TAG_DUPLICATE,
            },
        }
    }
}

pub enum FcntlCommand {
    // like F_DUPFD but it move fd insted of duplicating
    F_MVFD { min_fd_num: wasi::Fd },
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
) -> Result<(ExitCode, Pid), ExitCode> {
    let mut child_pid: Pid = -1;
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
        let redirects_len = redirects.len();
        let redirects_ptr = redirects
            .into_iter()
            .map(InternalRedirect::from)
            .collect::<Vec<InternalRedirect>>()
            .as_ptr();
        eprintln!("{:#?}", redirects_ptr);
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
            redirects_ptr as *const c_void,
            redirects_len,
            &mut child_pid,
        )
    };
    if syscall_result < 0 {
        Err(-syscall_result)
    } else {
        Ok((syscall_result, child_pid))
    }
}

pub fn kill(pid: Pid, signal: wasi::Signal) -> Result<(), ExitCode> {
    let result = unsafe { wasi_ext_lib_generated::wasi_ext_kill(pid, signal.raw() as i32) };
    if result < 0 {
        Err(-result)
    } else {
        Ok(())
    }
}

pub fn ioctl<T>(fd: RawFd, command: IoctlNum, arg: Option<&mut T>) -> Result<(), ExitCode> {
    let result = if let Some(arg) = arg {
        unsafe {
            let arg_ptr: *mut c_void = arg as *mut T as *mut c_void;
            wasi_ext_lib_generated::wasi_ext_ioctl(fd, command as c_uint, arg_ptr)
        }
    } else {
        unsafe {
            let null_ptr = ptr::null_mut::<T>() as *mut c_void;
            wasi_ext_lib_generated::wasi_ext_ioctl(fd, command as c_uint, null_ptr)
        }
    };

    if result < 0 {
        Err(-result)
    } else {
        Ok(())
    }
}
pub fn fcntl(fd: wasi::Fd, cmd: FcntlCommand) -> Result<i32, ExitCode> {
    match cmd {
        FcntlCommand::F_MVFD { min_fd_num } => {
            // Find free fd number not lower than min_fd_num
            let mut dst_fd = min_fd_num;
            loop {
                let result = unsafe { wasi::fd_fdstat_get(dst_fd) };
                if let Err(wasi::ERRNO_BADF) = result {
                    break;
                } else if let Err(err) = result {
                    return Err(err.raw() as ExitCode);
                }
                dst_fd += 1;
            }

            // Duplicate fd
            if let Err(err) = unsafe { wasi::fd_renumber(fd, dst_fd) } {
                return Err(err.raw() as ExitCode)
            }

            // Close fd
            if let Err(err) = unsafe { wasi::fd_close(fd) } {
                return Err(err.raw() as ExitCode)
            }

            Ok(dst_fd as i32)
        },
    }
}
