// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::Result;
use inapi::{ProviderFactory, Host};
use zdaemon::ZMsgExtended;

pub struct PackageApi;

impl PackageApi {
    pub fn default_provider(sock: &mut ZSock, host: &mut Host) -> Result<()> {
        let provider = try!(ProviderFactory::create(host, None));
        let msg = try!(ZMsg::new_ok());
        try!(msg.addstr(&provider.get_providers().to_string()));
        try!(msg.send(sock));
        Ok(())
    }
}
