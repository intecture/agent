// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use config::Config;

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct AgentConf {
    pub api_port: i32,
    pub upload_port: i32,
    pub download_port: i32,
}

impl Config for AgentConf {
    type ConfigFile = AgentConf;
}
