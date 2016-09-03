// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::Result;
use inapi::{Telemetry, Host};
use rustc_serialize::json;
use zdaemon::ZMsgExtended;

pub struct TelemetryApi;

impl TelemetryApi {
    pub fn get(sock: &mut ZSock, host: &mut Host) -> Result<()> {
        let telemetry = try!(Telemetry::init(host));
        let json = try!(json::encode(&telemetry));
        let msg = try!(ZMsg::new_ok());
        try!(msg.addstr(&json));
        try!(msg.send(sock));
        Ok(())
    }
}
