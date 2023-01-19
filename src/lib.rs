use std::collections::HashMap;
use std::str;
use std::env;

use serde_json::json;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

const EXIT_SUCCESS: i32 = 0;

type PID = u32;

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
) -> Result<String, wasi::Errno> {
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
                Ok(s) => String::from(s),
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
) -> Result<SyscallResult, String> {
    match syscall("spawn", &json!({
        "path": path,
        "args": args,
        "env": env,
        "redirects": redirects,
        "background": background,
        "working_dir": env::current_dir(),
    })) {
        Ok(result) => {
            if !background {
                let (exit_status, output) = result.split_once("\x1b").unwrap();
                let exit_status = exit_status.parse::<i32>().unwrap();
                Ok(SyscallResult {
                    exit_status,
                    output: output.to_string(),
                })
            } else {
                Ok(SyscallResult {
                    exit_status: EXIT_SUCCESS,
                    output: "".to_string(),
                })
            }
        },
        Err(e) => Err(String::from("Could not spawn process"))
    }
}
