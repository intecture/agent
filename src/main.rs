// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate czmq;
extern crate inapi;
extern crate rustc_serialize;
#[cfg(test)]
extern crate tempdir;

mod config;
mod error;
mod file;
mod handler;
mod msg;

use config::agent::AgentConf;
use czmq::{ZAuth, ZCert};
use error::Error;
use handler::{ApiHandler, FileHandler, Handler};
use std::fmt::Debug;
use std::fmt::Display;
use std::process::exit;
use std::result::Result as StdResult;
use std::sync::Arc;
use std::thread;

fn main() {
    let agent_conf = Arc::new(try_exit(AgentConf::load_path(None)));

    let zauth = try_exit(ZAuth::new());
    try_exit(zauth.load_curve(Some(&agent_conf.users_path)));

    let server_cert = Arc::new(try_exit(ZCert::load(&agent_conf.server_cert)));

    let api = ApiHandler::new(agent_conf.clone(), server_cert.clone());
    let api_thread = thread::spawn(move || {
        // XXX This error should be logged.
        let _ = api.run();
    });

    let file = FileHandler::new(agent_conf.clone(), server_cert.clone());
    let file_thread = thread::spawn(move || {
        // XXX This error should be logged.
        let _ = file.run();
    });

    file_thread.join().unwrap();
    api_thread.join().unwrap();
}

fn try_exit<T, E>(r: StdResult<T, E>) -> T
    where E: Into<Error> + Debug + Display {
    if let Err(e) = r {
        println!("{}", e);
        exit(1);
    }

    r.unwrap()
}
