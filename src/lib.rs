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
use std::ffi::{c_int, c_uint, c_void, CString};
use std::fs;
use std::io;
use std::mem;
use std::os::fd::AsRawFd;
use std::os::fd::RawFd;
use std::os::wasi::ffi::OsStrExt;
use std::path::Path;
use std::ptr;
use std::str;

mod wasi_ext_lib_generated;
use wasi_ext_lib_generated::{
    RedirectType_APPEND, RedirectType_CLOSE, RedirectType_DUPLICATE, RedirectType_PIPEIN,
    RedirectType_PIPEOUT, RedirectType_READ, RedirectType_READWRITE, RedirectType_WRITE,
    Redirect_Data, Redirect_Data_Path,
};

pub use wasi_ext_lib_generated::{
    WASI_EXT_FDFLAG_CLOEXEC, WASI_EXT_FDFLAG_CTRL_BIT, WASI_EXT_FDFLAG_MASK,
};

#[cfg(feature = "hterm")]
pub use wasi_ext_lib_generated::{
    WasiEvents, WASI_EVENTS_MASK_SIZE, WASI_EVENTS_NUM, WASI_EVENT_SIGINT, WASI_EVENT_WINCH,
};

// #[cfg(feature = "hterm")]
// use wasi_ext_lib_generated::{TIOCGWINSZ, TIOCSECHO, TIOCSRAW};
// Bindgen cannot properly expand functional macros to generate constants
// from macros. These constants need to be hard-coded for now.
// See https://github.com/rust-lang/rust-bindgen/issues/753
const TIOCGWINSZ: u32 = 2148008192;
const TIOCSRAW: u32 = 2147746049;
const TIOCSECHO: u32 = 2147746050;

pub use wasi::SIGNAL_KILL;

type ExitCode = i32;
type Pid = i32;

#[derive(Debug)]
pub enum Redirect {
    Read(wasi::Fd, String),
    Write(wasi::Fd, String),
    Append(wasi::Fd, String),
    ReadWrite(wasi::Fd, String),
    PipeIn(wasi::Fd),
    PipeOut(wasi::Fd),
    Duplicate { fd_src: wasi::Fd, fd_dst: wasi::Fd },
    Close(wasi::Fd),
}

#[repr(u32)]
pub enum IoctlNum {
    GetScreenSize = TIOCGWINSZ,
    SetRaw = TIOCSRAW,
    SetEcho = TIOCSECHO,
}

impl From<&Redirect> for wasi_ext_lib_generated::Redirect {
    fn from(redirect: &Redirect) -> Self {
        match redirect {
            Redirect::Read(fd, path)
            | Redirect::Write(fd, path)
            | Redirect::Append(fd, path)
            | Redirect::ReadWrite(fd, path) => {
                let tag = match redirect {
                    Redirect::Read(_, _) => RedirectType_READ,
                    Redirect::Write(_, _) => RedirectType_WRITE,
                    Redirect::Append(_, _) => RedirectType_APPEND,
                    Redirect::ReadWrite(_, _) => RedirectType_READWRITE,
                    _ => unreachable!(),
                };

                wasi_ext_lib_generated::Redirect {
                    data: Redirect_Data {
                        path: Redirect_Data_Path {
                            path_str: path.as_ptr() as *const i8,
                            path_len: path.len(),
                        },
                    },
                    fd_dst: *fd as i32,
                    type_: tag,
                }
            }
            Redirect::PipeIn(fd_src) => wasi_ext_lib_generated::Redirect {
                data: Redirect_Data {
                    fd_src: *fd_src as i32,
                },
                fd_dst: io::stdin().as_raw_fd(),
                type_: RedirectType_PIPEIN,
            },
            Redirect::PipeOut(fd_src) => wasi_ext_lib_generated::Redirect {
                data: Redirect_Data {
                    fd_src: *fd_src as i32,
                },
                fd_dst: io::stdout().as_raw_fd(),
                type_: RedirectType_PIPEOUT,
            },
            Redirect::Duplicate { fd_src, fd_dst } => wasi_ext_lib_generated::Redirect {
                data: Redirect_Data {
                    fd_src: *fd_src as i32,
                },
                fd_dst: *fd_dst as i32,
                type_: RedirectType_DUPLICATE,
            },
            Redirect::Close(fd_dst) => wasi_ext_lib_generated::Redirect {
                data: unsafe { mem::zeroed() }, // ignore field in kernel
                fd_dst: *fd_dst as i32,
                type_: RedirectType_CLOSE,
            },
        }
    }
}

pub enum FcntlCommand {
    // like F_DUPFD but it move fd insted of duplicating
    F_MVFD { min_fd_num: wasi::Fd },
    F_GETFD,
    F_SETFD { flags: wasi::Fdflags },
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
        unsafe { wasi_ext_lib_generated::wasi_ext_set_env(c_key.as_ptr(), c_val.as_ptr()) }
    } else {
        unsafe { wasi_ext_lib_generated::wasi_ext_set_env(c_key.as_ptr(), ptr::null::<i8>()) }
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
    redirects: &[Redirect],
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
        let redirects_vec = redirects
            .iter()
            .map(wasi_ext_lib_generated::Redirect::from)
            .collect::<Vec<wasi_ext_lib_generated::Redirect>>();
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
            redirects_vec.as_ptr(),
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
    let result = match cmd {
        FcntlCommand::F_MVFD { min_fd_num } => unsafe {
            let mut min_fd = min_fd_num;
            wasi_ext_lib_generated::wasi_ext_fcntl(
                fd as c_int,
                wasi_ext_lib_generated::FcntlCommand_F_MVFD,
                (&mut min_fd as *mut u32) as *mut c_void,
            )
        },
        FcntlCommand::F_GETFD => unsafe {
            let null_ptr = ptr::null_mut::<c_void>();
            wasi_ext_lib_generated::wasi_ext_fcntl(
                fd as c_int,
                wasi_ext_lib_generated::FcntlCommand_F_GETFD,
                null_ptr,
            )
        },
        FcntlCommand::F_SETFD { flags } => unsafe {
            let mut flags = flags;
            wasi_ext_lib_generated::wasi_ext_fcntl(
                fd as c_int,
                wasi_ext_lib_generated::FcntlCommand_F_SETFD,
                (&mut flags as *mut wasi::Fdflags) as *mut c_void,
            )
        },
    };

    if result < 0 {
        Err(-result)
    } else {
        Ok(result)
    }
}

pub fn mount(
    source_path: &str,
    target_path: &str,
    filesystem_type: &str,
    opts: u64,
    data: &str,
) -> Result<(), ExitCode> {
    let c_source_path = CString::new(source_path).unwrap();
    let c_target_path = CString::new(target_path).unwrap();

    let c_filesystem_type = CString::new(filesystem_type).unwrap();
    let c_data = CString::new(data).unwrap();

    let result = unsafe {
        wasi_ext_lib_generated::wasi_ext_mount(
            -1,
            c_source_path.as_ptr() as *const i8,
            -1,
            c_target_path.as_ptr() as *const i8,
            c_filesystem_type.as_ptr() as *const i8,
            opts,
            c_data.as_ptr() as *const i8,
        )
    };

    if result == 0 {
        Ok(())
    } else {
        Err(result)
    }
}
