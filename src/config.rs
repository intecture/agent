// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub api_port: u32,
    pub server_cert: String,
    pub filexfer_port: u32,
    pub filexfer_threads: u32,
    pub auth_server: String,
    pub auth_update_port: u32,
    pub auth_cert: String,
}
