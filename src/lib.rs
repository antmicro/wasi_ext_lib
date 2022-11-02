use std::collections::HashMap;
use std::str;
use std::env;
use std::fs;
use std::ptr;
use std::path::Path;
use std::convert::AsRef;
use std::ffi::CString;
use std::os::wasi::ffi::OsStrExt;

mod wasi_ext_lib_generated;

type ExitCode = i32;
type Pid = i32;

pub enum Redirect {
    Read((wasi::Fd, String)),
    Write((wasi::Fd, String)),
    Append((wasi::Fd, String)),
}

unsafe fn get_c_redirect(r: &Redirect) -> wasi_ext_lib_generated::Redirect{
    match r {
        Redirect::Read((fd, path)) => { wasi_ext_lib_generated::Redirect {
            type_: wasi_ext_lib_generated::RedirectType_READ,
            path: CString::new(&path[..]).unwrap().as_c_str().as_ptr(),
            fd: *fd as i32
        }},
        Redirect::Write((fd, path)) => { wasi_ext_lib_generated::Redirect {
            type_: wasi_ext_lib_generated::RedirectType_WRITE,
            path: CString::new(&path[..]).unwrap().as_c_str().as_ptr(),
            fd: *fd as i32
        }},
        Redirect::Append((fd, path)) => { wasi_ext_lib_generated::Redirect {
            type_: wasi_ext_lib_generated::RedirectType_APPEND,
            path: CString::new(&path[..]).unwrap().as_c_str().as_ptr(),
            fd: *fd as i32
        }},
    }
}

pub struct SyscallResult {
    pub exit_status: i32,
    pub output: String,
}


pub fn chdir<P: AsRef<Path>>(path: P) -> Result<(), ExitCode> {
    if let Ok(canon) = fs::canonicalize(&path) {
        if let Err(_) = env::set_current_dir(&canon) {
            return Err(wasi::ERRNO_NOENT.raw().into())
        };
        let pth = match CString::new(path.as_ref().as_os_str().as_bytes()) {
            Ok(p) => p,
            Err(_) => { return Err(wasi::ERRNO_INVAL.raw().into()) }
        };
        match unsafe { wasi_ext_lib_generated::wasi_ext_chdir(pth.as_ptr()) } {
            0 => Ok(()),
            e => Err(e)
        }
    } else {
        Err(wasi::ERRNO_INVAL.raw().into())
    }
}

pub fn getcwd() -> Result<String, ExitCode> {
    const BUF_LEN: usize = 256;
    let mut buf = [0u8; BUF_LEN];
    match unsafe { wasi_ext_lib_generated::wasi_ext_getcwd(buf.as_mut_ptr() as *mut i8, BUF_LEN) } {
        0 => Ok(std::str::from_utf8(&buf).expect("Could not read syscall output").to_string()),
        e => Err(e)
    }
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
    match unsafe { wasi_ext_lib_generated::wasi_ext_set_env(key.as_ptr() as *const i8, match val {
        Some(v) => v.as_ptr() as *mut i8,
        None => ptr::null::<i8>()
    })} {
        0 => Ok(()),
        e => Err(e)
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
        e => Err(e)
    }
}

#[cfg(feature = "hterm")]
pub fn hterm(attrib: &str, val: Option<&str>) -> Result<Option<String>, ExitCode> {
    match val {
        Some(value) => {
            match unsafe {
                wasi_ext_lib_generated::wasi_ext_hterm_set(
                    CString::new(&attrib[..]).unwrap().as_c_str().as_ptr() as *const i8,
                    CString::new(&value[..]).unwrap().as_c_str().as_ptr() as *const i8
                )
            } {
                0 => Ok(None),
                e => Err(e)
            }
        },
        None => {
            const output_len: usize = 256;
            let mut buf = [0u8; output_len];
            match unsafe {
                wasi_ext_lib_generated::wasi_ext_hterm_get(
                    CString::new(&attrib[..]).unwrap().as_c_str().as_ptr() as *const i8,
                    buf.as_mut_ptr() as *mut i8,
                    output_len
                )
            } {
                0 => Ok(Some(str::from_utf8(&buf).expect("Could not read syscall output").to_string())),
                e => Err(e)
            }
        }
    }
}

pub fn spawn(
    path: &str,
    args: &[&str],
    env: &HashMap<String, String>,
    background: bool,
    redirects: &[Redirect]
) -> Result<ExitCode, ExitCode> {
    let syscall_result = unsafe {
        wasi_ext_lib_generated::wasi_ext_spawn(
            CString::new(path).unwrap().as_c_str().as_ptr(),
            args.iter().map(|arg| {
                CString::new(*arg).unwrap().as_c_str().as_ptr()
            }).collect::<Vec<*const i8>>().as_ptr(),
            args.len(),
            env.iter().map(|(key, val)| {
                wasi_ext_lib_generated::Env {
                    attrib: CString::new(&key[..]).unwrap().as_c_str().as_ptr(),
                    val: CString::new(&val[..]).unwrap().as_c_str().as_ptr()
                }
            }).collect::<Vec<wasi_ext_lib_generated::Env>>().as_ptr(),
            env.len(),
            background as i32,
            redirects.iter().map(|red| {
                get_c_redirect(red)
            }).collect::<Vec<wasi_ext_lib_generated::Redirect>>().as_ptr(),
            redirects.len()
        )
    };
    if syscall_result < 0 {
        Err(-syscall_result)
    } else {
        Ok(syscall_result)
    }
}
