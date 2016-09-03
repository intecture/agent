// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate chan;
extern crate chan_signal;
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

use chan_signal::Signal;
use config::Config;
use czmq::{ZCert, ZSock, ZSockType, ZSys};
use error::Result;
use inauth_client::{CertType, ZapHandler};
use std::process::exit;
use std::thread::spawn;
use zdaemon::{ConfigFile, Service};
use zfilexfer::Server as FileServer;

fn main() {
    if let Err(e) = start() {
        println!("{}", e);
        exit(1);
    }
}

fn start() -> Result<()> {
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (parent, child) = try!(ZSys::create_pipe());

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

    let mut file_sock = ZSock::new(ZSockType::ROUTER);
    server_cert.apply(&mut file_sock);
    file_sock.set_zap_domain("agent.intecture");
    file_sock.set_curve_server(true);
    file_sock.set_linger(1000);
    try!(file_sock.bind(&format!("tcp://*:{}", config.filexfer_port)));

    let thread = spawn(move || {
        let mut service = Service::new(child).unwrap();

        let api_endpoint = api::endpoint(config.api_port, &server_cert).unwrap();
        service.add_endpoint(api_endpoint).unwrap();

        let file_endpoint = FileServer::new(file_sock, config.filexfer_threads).unwrap();
        service.add_endpoint(file_endpoint).unwrap();

        service.start(None).unwrap();
    });

    // Wait for interrupt from system
    signal.recv().unwrap();

    // Terminate loop
    try!(parent.signal(1));
    thread.join().unwrap();

    Ok(())
}
