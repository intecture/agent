// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use zdaemon::ConfigFile;

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct Config {
    pub auth_server: String,
    pub auth_server_port: u32,
    pub auth_server_cert: String,
    pub server_cert: String,
    pub api_port: u32,
    pub filexfer_port: u32,
    pub filexfer_threads: u32,
}

impl ConfigFile for Config {}
