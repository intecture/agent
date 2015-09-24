use std::process::{Command, Output};
use std::io;

pub fn exec<'a>(cmd: &str) -> io::Result<Output> {
    Command::new("sh").arg("-c").arg(cmd).output()
}