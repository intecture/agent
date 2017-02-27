// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate chan;
extern crate chan_signal;
extern crate czmq;
extern crate docopt;
extern crate inapi;
extern crate inauth_client;
extern crate rustc_serialize;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[cfg(test)]
extern crate tempdir;
extern crate zdaemon;
extern crate zfilexfer;

mod api;
mod config;
mod error;

use chan_signal::Signal;
use config::Config;
use czmq::{ZCert, ZSock, SocketType, ZSys};
use docopt::Docopt;
use error::Result;
use inauth_client::{CertType, ZapHandler};
use std::{env, fs};
use std::io::Read;
use std::path::Path;
use std::process::exit;
use std::thread::spawn;
use zdaemon::Service;
use zfilexfer::Server as FileServer;

static USAGE: &'static str = "
Intecture Agent.

Usage:
  inagent [(-c <path> | --config <path>)]
  inagent (-h | --help)
  inagent --version

Options:
  -c --config <path>    Path to agent.json, e.g. \"/usr/local/etc\"
  -h --help             Show this screen.
  --version             Print this script's version.
";

#[derive(Debug, RustcDecodable)]
#[allow(non_snake_case)]
struct Args {
    flag_c: Option<String>,
    flag_config: Option<String>,
    flag_h: bool,
    flag_help: bool,
    flag_version: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e| e.exit());

    if args.flag_version {
        println!(env!("CARGO_PKG_VERSION"));
        exit(0);
    } else {
        let config_path = if args.flag_c.is_some() { args.flag_c.as_ref() } else { args.flag_config.as_ref() };
        if let Err(e) = start(config_path) {
            println!("{}", e);
            exit(1);
        }
    }
}

fn start<P: AsRef<Path>>(path: Option<P>) -> Result<()> {
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    let (parent, child) = try!(ZSys::create_pipe());
    parent.set_linger(0);
    parent.set_sndtimeo(Some(100));

    let config = read_conf(path)?;
    let server_cert = try!(ZCert::load(&config.server_cert));
    let auth_cert = try!(ZCert::load(&config.auth_cert));

    let _auth = ZapHandler::new(
        Some(CertType::User),
        &server_cert,
        &auth_cert,
        &config.auth_server,
        config.auth_update_port,
        false);

    let mut file_sock = ZSock::new(SocketType::ROUTER);
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

fn read_conf<P: AsRef<Path>>(path: Option<P>) -> Result<Config> {
    if let Some(p) = path {
        do_read_conf(p)
    }
    else if let Ok(p) = env::var("INAGENT_CONFIG_DIR") {
        do_read_conf(p)
    }
    else if let Ok(c) = do_read_conf("/usr/local/etc/intecture") {
        Ok(c)
    } else {
        do_read_conf("/etc/intecture")
    }
}

fn do_read_conf<P: AsRef<Path>>(path: P) -> Result<Config> {
    let mut path = path.as_ref().to_owned();
    path.push("agent.json");

    let mut fh = fs::File::open(&path)?;
    let mut json = String::new();
    fh.read_to_string(&mut json)?;
    Ok(serde_json::from_str(&json)?)
}
