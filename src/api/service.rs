// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::{Error, Result};
use inapi::{Service, ServiceRunnable, Host};
use zdaemon::ZMsgExtended;

pub struct ServiceApi;

impl ServiceApi {
    pub fn action(sock: &mut ZSock, host: &mut Host, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;
        let runnable = request.popstr().unwrap().or(Err(Error::MessageUtf8))?;
        let service = Service::new_service(ServiceRunnable::Service(&runnable), None);
        let result = service.action(host, &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;

        let msg = ZMsg::new_ok(Some(router_id))?;
        if let Some(r) = result {
            msg.send_multi(sock, &[
                &r.exit_code.to_string(),
                &r.stdout,
                &r.stderr,
            ])?;
        } else {
            msg.send(sock)?;
        }
        Ok(())
    }
}
