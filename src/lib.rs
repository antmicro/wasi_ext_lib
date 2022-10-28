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

pub struct SyscallResult {
    pub exit_status: i32,
    pub output: String,
}

impl Serialize for Redirect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut state = serializer.serialize_struct("Redirect", 3)?;
        match self {
            Redirect::Read((fd, path)) => {
                state.serialize_field("mode", "read")?;
                state.serialize_field("fd", fd)?;
                state.serialize_field("path", path)?;
            }
            Redirect::Write((fd, path)) => {
                state.serialize_field("mode", "write")?;
                state.serialize_field("fd", fd)?;
                state.serialize_field("path", path)?;
            }
            Redirect::Append((fd, path)) => {
                state.serialize_field("mode", "append")?;
                state.serialize_field("fd", fd)?;
                state.serialize_field("path", path)?;
            }
        }
        state.end()
    }
}


            }
        }
    })
}

pub fn spawn(
    path: &str,
    args: &[&str],
    env: &HashMap<String, String>,
    background: bool,
    redirects: &[Redirect]
) -> SyscallResult {
    match syscall("spawn", &json!({
        "path": path,
        "args": args,
        "env": env,
        "redirects": redirects,
        "background": background,
        "working_dir": env::current_dir().unwrap_or(PathBuf::from("/")),
    })) {
        Ok(result) => result,
        Err(e) => SyscallResult {
            exit_status: e.raw().into(),
            output: String::from("Could not invoke syscall")
        }
    }
}


pub fn getcwd() -> Result<String, ExitCode> {
    match syscall("getcwd", &json!({"buf_len": 1024})) {
        Ok(result) => {
            if let 0 = result.exit_status {
                Ok(result.output)
            } else {
                Err(result.exit_status)
            }
        }
        Err(e) => Err(e.raw().into()),
    }
}

pub fn isatty(fd: wasi::Fd) -> Result<bool, ExitCode> {
    match syscall("isatty", &json!({ "fd": fd })) {
        Ok(result) => {
            if let 0 = result.exit_status {
                Ok(match result.output.as_str() {
                    "0" => false,
                    "1" => true,
                    _ => return Err(wasi::ERRNO_BADMSG.raw().into())
                })
            } else {
                Err(result.exit_status)
            }
        },
        Err(e) => return Err(e.raw().into())
    }
}

pub fn getpid() -> Result<Pid, ExitCode> {
    match syscall("getpid", &json!({})) {
        Ok(result) => {
            if let 0 = result.exit_status {
                if let Ok(a) = result.output.parse::<u32>() {
                    Ok(a)
                } else {
                    Err(wasi::ERRNO_BADMSG.raw().into())
                }
            } else {
                Err(result.exit_status)
            }
        },
        Err(e) => Err(e.raw().into())
    }
}

pub fn set_env(key: &str, val: Option<&str>) -> Result<(), ExitCode> {
    match syscall("set_env", &if let Some(value) = val {
        json!({
            "key": key,
            "value": value
        })
    } else {
        json!({
            "key": key,
        })
    }) {
        Ok(result) => {
            if let 0 = result.exit_status {
                Ok(())
            } else {
                Err(result.exit_status)
            }
        }
        Err(e) => Err(e.raw().into())
    }
}

pub fn set_echo(should_echo: bool) -> Result<(), ExitCode> {
    match syscall("set_echo", &json!({"echo": should_echo})) {
        Ok(result) => {
            if let 0 = result.exit_status {
                Ok(())
            } else {
                Err(result.exit_status)
            }
        }
        Err(e) => Err(e.raw().into())
    }
}

#[cfg(feature = "hterm")]
pub fn hterm(attrib: &str, val: Option<&str>) -> Result<Option<String>, ExitCode> {
    match val {
        Some(value) => {
            match syscall("hterm", &json!({ "method": "set", "attrib": attrib, "val": value })) {
                Ok(result) => {
                    if let 0 = result.exit_status {
                        Ok(None)
                    } else {
                        Err(result.exit_status)
                    }
                }
                Err(e) => Err(e.raw().into())
            }
        },
        None => {
            match syscall("hterm", &json!({ "method": "get", "attrib": attrib })) {
                Ok(result) => {
                    if let 0 = result.exit_status {
                        Ok(Some(result.output))
                    } else {
                        Err(result.exit_status)
                    }
                }
                Err(e) => Err(e.raw().into())
            }
        }
    }
}
