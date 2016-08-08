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
use error::Error;
use inauth_client::{CertType, ZapHandler};
use std::fmt::{Debug, Display};
use std::process::exit;
use std::result::Result as StdResult;
use zdaemon::Service;
use zfilexfer::Server as FileServer;

fn main() {
    let mut service: Service<Config> = try_exit(Service::load("agent.json"));
    let server_cert = try_exit(ZCert::load(&service.get_config().unwrap().server_cert));
    let auth_cert = try_exit(ZCert::load(&service.get_config().unwrap().auth_server_cert));

    let _auth = ZapHandler::new(
        Some(CertType::User),
        &server_cert,
        &auth_cert,
        &service.get_config().unwrap().auth_server,
        service.get_config().unwrap().auth_server_port,
        false);

    let api_endpoint = try_exit(api::endpoint(service.get_config().unwrap().api_port, &server_cert));
    try_exit(service.add_endpoint(api_endpoint));

    let file_sock = ZSock::new(ZSockType::REP);
    server_cert.apply(&file_sock);
    file_sock.set_zap_domain("agent.intecture");
    file_sock.set_curve_server(true);
    try_exit(file_sock.bind(&format!("tcp://*:{}", service.get_config().unwrap().filexfer_port)));

    let file_endpoint = try_exit(FileServer::new(file_sock, service.get_config().unwrap().filexfer_threads));
    try_exit(service.add_endpoint(file_endpoint));

    try_exit(service.start(None));
}

fn try_exit<T, E>(r: StdResult<T, E>) -> T
    where E: Into<Error> + Debug + Display {
    if let Err(e) = r {
        println!("{}", e);
        exit(1);
    }

    r.unwrap()
}
