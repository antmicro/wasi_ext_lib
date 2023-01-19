use std::collections::HashMap;

use serde_json::json;
use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

const EXIT_SUCCESS: i32 = 0;

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

pub fn syscall(
    command: &str,
    args: &[&str],
    env: &HashMap<String, String>,
    background: bool,
    redirects: &[Redirect],
) -> std::io::Result<SyscallResult> {
    let result = {
        let working_dir = match std::env::current_dir() {
            Ok(path) => {
                path.display().to_string()
            },
            Err(e) => {
                eprintln!("Parsing current directory path error: {}", e);
                String::from("/")
            },
        };

        let j = json!({
            "args": args,
            "env": env,
            "redirects": redirects,
            "background": background,
            "working_dir": working_dir,
        }).to_string();
        let c = json!({
            "command": command,
            "buf_len": j.len(),
            "buf_ptr": format!("{:?}", j.as_ptr()),
        }).to_string();
        let result = std::fs::read_link(format!("/!{}", c))?
            .to_str()
            .unwrap()
            .trim_matches(char::from(0))
            .to_string();
        if !background {
            let (exit_status, output) = result.split_once("\x1b").unwrap();
            let exit_status = exit_status.parse::<i32>().unwrap();
            SyscallResult {
                exit_status,
                output: output.to_string(),
            }
        } else {
            SyscallResult {
                exit_status: EXIT_SUCCESS,
                output: "".to_string(),
            }
        }
    };
    Ok(result)
}
