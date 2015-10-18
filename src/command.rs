// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use std::process::{Command, Output};
use std::io;

pub fn exec<'a>(cmd: &str) -> io::Result<Output> {
    Command::new("sh").arg("-c").arg(cmd).output()
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    #[test]
    fn test_exec() {
        let cmd = super::exec("whoami").unwrap();
        let user = Command::new("whoami").output().unwrap();

        assert_eq!(cmd.status.code().unwrap(), 0);
        assert_eq!(String::from_utf8(cmd.stdout).unwrap(), String::from_utf8(user.stdout).unwrap());
    }
}
