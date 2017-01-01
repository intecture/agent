// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::{Error, Result};
use inapi::{Command, Host};
use zdaemon::ZMsgExtended;

pub struct CommandApi;

impl CommandApi {
    pub fn exec(sock: &mut ZSock, host: &mut Host, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let cmd = Command::new(&request.popstr().unwrap().or(Err(Error::MessageUtf8))?);
        let result = cmd.exec(host)?;

        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.send_multi(sock, &[
            &result.exit_code.to_string(),
            &result.stdout,
            &result.stderr
        ])?;

        Ok(())
    }
}
