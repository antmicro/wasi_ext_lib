use std::collections::HashMap;
use std::str;
use std::env;
use std::path::PathBuf;

use serde_json::json;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

type ExitCode = i32;
type Pid = u32;

pub enum Redirect {
    Read((wasi::Fd, String)),
    Write((wasi::Fd, String)),
    Append((wasi::Fd, String)),
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

fn syscall(
    command: &str,
    data: &serde_json::Value
) -> Result<SyscallResult, wasi::Errno> {
    Ok({
        let j = data.to_string();
        let c = json!({
            "command": command,
            "buf_len": j.len(),
            "buf_ptr": format!("{:?}", j.as_ptr()),
        }).to_string();

        const BUF_LEN: usize = 1024;
        let mut buf = vec![0u8; BUF_LEN];
        unsafe {
            let result_len = wasi::path_readlink(4, &format!("/!{}", c), buf.as_mut_ptr(), BUF_LEN)?;
            match str::from_utf8(&buf[0..result_len]) {
                Ok(result) => {
                    if let Some((exit_status, output)) = result.split_once("\x1b") {
                        SyscallResult{
                            exit_status: if let Ok(n) = exit_status.parse::<i32>() {
                                n
                            } else {
                                return Err(wasi::ERRNO_BADMSG)
                            },
                            output: output.to_string()
                        }
                    } else {
                        return Err(wasi::ERRNO_BADMSG)
                    }
                }
                Err(_) => return Err(wasi::ERRNO_BADMSG)
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
) -> Result<(), ExitCode> {
    match syscall("spawn", &json!({
        "path": path,
        "args": args,
        "env": env,
        "redirects": redirects,
        "background": background,
        "working_dir": env::current_dir().unwrap_or(PathBuf::from("/")),
    })) {
        Ok(result) => {
            if let 0 = result.exit_status {
                Ok(())
            } else {
                Err(result.exit_status)
            }
        },
        Err(e) => Err(e.raw().into())
    }
}

pub fn chdir(path: &str) -> Result<(), ExitCode> {
    match std::env::set_current_dir(path) {
        Ok(()) => (),
        Err(_) => return Err(wasi::ERRNO_NOENT.raw().into())
    };
    match syscall("chdir", &json!({ "dir": path })) {
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

pub fn getcwd() -> Result<String, ExitCode> {
    match syscall("getcwd", &json!({})) {
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

pub fn set_env(key: &str, value: &str) -> Result<(), ExitCode> {
    match syscall("set_env", &json!({
        "key": key,
        "value": value
    })) {
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
