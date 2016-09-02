// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate czmq;
extern crate inapi;
extern crate inauth_client;
extern crate rustc_serialize;
#[cfg(test)]
extern crate tempdir;
extern crate zdaemon;
extern crate zfilexfer;

mod api;
mod config;
mod error;

use config::Config;
use czmq::{ZCert, ZSock, ZSockType};
use error::Result;
use inauth_client::{CertType, ZapHandler};
use std::process::exit;
use zdaemon::{ConfigFile, Service};
use zfilexfer::Server as FileServer;

fn main() {
    if let Err(e) = start() {
        println!("{}", e);
        exit(1);
    }
}

fn start() -> Result<()> {
    let mut service = try!(Service::new());
    let config = try!(Config::search("intecture/agent.json", None));
    let server_cert = try!(ZCert::load(&config.server_cert));
    let auth_cert = try!(ZCert::load(&config.auth_server_cert));

    let _auth = ZapHandler::new(
        Some(CertType::User),
        &server_cert,
        &auth_cert,
        &config.auth_server,
        config.auth_server_port,
        false);

    let api_endpoint = try!(api::endpoint(config.api_port, &server_cert));
    try!(service.add_endpoint(api_endpoint));

    let file_sock = ZSock::new(ZSockType::ROUTER);
    server_cert.apply(&file_sock);
    file_sock.set_zap_domain("agent.intecture");
    file_sock.set_curve_server(true);
    file_sock.set_linger(1000);
    try!(file_sock.bind(&format!("tcp://*:{}", config.filexfer_port)));

    let file_endpoint = try!(FileServer::new(file_sock, config.filexfer_threads));
    try!(service.add_endpoint(file_endpoint));

    try!(service.start(true, None));
    Ok(())
}
