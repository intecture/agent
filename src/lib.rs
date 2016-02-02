// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate rustc_serialize;

mod config;
pub mod file;

pub use config::agent::AgentConf;
pub use config::Config;
use std::path::PathBuf;

pub fn load_agent_conf<'a>() -> Result<AgentConf, AgentError> {
    for p in ["/usr/local/etc", "/etc"].iter() {
        let mut path = PathBuf::from(p);
        path.push("intecture");
        path.push("agent.json");

        match AgentConf::load(&path) {
            Ok(conf) => return Ok(conf),
            Err(_) => continue,
        }
    }

    Err(AgentError::MissingConf)
}

#[derive(Debug)]
pub enum AgentError {
    MissingConf,
}
