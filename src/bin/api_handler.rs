// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate inagent;
extern crate inapi;
extern crate rustc_serialize;
extern crate zmq;

use inagent::{AgentConf, load_agent_conf};
use inapi::{Command, File, Host, ProviderFactory, Telemetry};
use rustc_serialize::json;
use std::error::Error;
use std::process::exit;
use std::result;

pub type Result<T> = result::Result<T, ApiError>;

#[derive(Debug)]
pub enum ApiError {
    TooManyArgs,
}

fn main() {
    let mut ctx = zmq::Context::new();

    // Load agent config
    let agent_conf: AgentConf;
    match load_agent_conf() {
        Ok(conf) => agent_conf = conf,
        Err(e) => {
            println!("{:?}", e);
            exit(1);
        },
    }

    let mut api_sock = ctx.socket(zmq::REP).unwrap();
    api_sock.bind(&format!("tcp://*:{}", agent_conf.api_port)).unwrap();

    let mut file_sock = ctx.socket(zmq::PAIR).unwrap();
    file_sock.bind("inproc:///tmp/inagent.sock").unwrap();

    let mut host = Host::new();

    loop {
        let endpoint_msg = api_sock.recv_msg(0).unwrap();
        let endpoint = endpoint_msg.as_str().unwrap();

        match endpoint {
            "command::exec" => {
                let args;

                match recv_args(&mut api_sock, 1, 1) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                let cmd = Command::new(&args[0]);

                match cmd.exec(&mut host) {
                    Ok(result) => send_args(&mut api_sock, vec![
                        "Ok",
                        &result.exit_code.to_string(),
                        &result.stdout,
                        &result.stderr,
                    ]),
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::is_file" => {
                let args;

                match recv_args(&mut api_sock, 1, 1) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(_) => send_args(&mut api_sock, vec!["Ok", "1"]),
                    Err(_) => send_args(&mut api_sock, vec!["Ok", "0"]),
                }
            },
            "file::exists" => {
                let args;

                match recv_args(&mut api_sock, 1, 1) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.exists(&mut host) {
                            Ok(exists) => send_args(&mut api_sock, vec!["Ok", if exists { "1" } else { "0" }]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::upload" => {
                let args;

                match recv_args(&mut api_sock, 4, 4) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                send_args(&mut file_sock, args.iter().map(|a| a.as_ref()).collect());

                match recv_args(&mut file_sock, 1, 1) {
                    Ok(r) => match r.first().unwrap().as_ref() {
                        "Ok" => send_args(&mut api_sock, vec![ "Ok" ]),
                        "Err" => send_args(&mut api_sock, vec![ "Err" ]),
                        _ => send_args(&mut api_sock, vec![ "Err", "Unexpected response" ]),
                    },
                    Err(_) => send_args(&mut api_sock, vec!["Err", "Could not recv from file handler"]),
                }
            },
            "package::default_provider" => {
                if recv_args(&mut api_sock, 0, 0).is_err() {
                    continue;
                }

                match ProviderFactory::create(&mut host, None) {
                    Ok(provider) => send_args(&mut api_sock, vec![
                        "Ok",
                        &provider.get_providers().to_string(),
                    ]),
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "telemetry" => {
                if recv_args(&mut api_sock, 0, 0).is_err() {
                    continue;
                }

                match Telemetry::init(&mut host) {
                    Ok(telemetry) => {
                        let json = json::encode(&telemetry);
                        if json.is_err() {
                            send_args(&mut api_sock, vec!["Err", json.unwrap_err().description() ])
                        } else {
                            send_args(&mut api_sock, vec!["Ok", &json.unwrap() ]);
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description() ]),
                }
            },
            _ => {
                api_sock.send_str("Err", zmq::SNDMORE).unwrap();
                api_sock.send_str(&format!("Invalid endpoint {}", endpoint), 0).unwrap();
                continue;
            }
        }
    }
}

/// Receive a variable number of args from sock. We expect the number
/// of args from `sock` to be between `min` and `max`.
/// If max = 0 then we allow a variable number of args.
fn recv_args(sock: &mut zmq::Socket, min: u8, max: u8) -> Result<Vec<String>> {
    let mut args: Vec<String> = vec![];
    let mut counter = 0;

    // Always receive all args, otherwise our socket will be in the
    // wrong state to send messages.
    while sock.get_rcvmore().unwrap() == true {
        args.push(sock.recv_string(0).unwrap().unwrap());
        counter += 1;
    }

    if min > counter || (max < counter && max > 0) {
        sock.send_str("Err", zmq::SNDMORE).unwrap();
        sock.send_str("Missing argument", 0).unwrap();
        return Err(ApiError::TooManyArgs);
    }

    Ok(args)
}

fn send_args(sock: &mut zmq::Socket, args: Vec<&str>) {
    let iter = args.iter();
    let iter_len = iter.len();
    let mut x = 1;
    let mut flag;
    for msg in iter {
        if x < iter_len {
            flag = zmq::SNDMORE;
        } else {
            flag = 0;
        }

        sock.send_str(&msg, flag).unwrap();
        x += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::{recv_args, send_args};
    use zmq;

    /// Test providing 1 and only 1 arg
    #[test]
    fn test_recv_args_ok_eq() {
        let mut ctx = zmq::Context::new();

        let mut rep_sock = ctx.socket(zmq::REP).unwrap();
        rep_sock.bind("inproc://test_recv_args").unwrap();

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_recv_args").unwrap();

        req_sock.send_str("0", zmq::SNDMORE).unwrap();
        req_sock.send_str("1", 0).unwrap();

        // Eliminate race condition where REP socket prematurely
        // gives up receiving and tries to send err while in
        // incorrect state. This only happens in the test because we
        // call `recv_args()` out of context.
        let _ = rep_sock.recv_string(0).unwrap();

        let result = recv_args(&mut rep_sock, 1, 1);
        assert_eq!(result.unwrap(), vec!["1"]);
    }

    /// Test providing 1 or more args
    #[test]
    fn test_recv_args_ok_range() {
        let mut ctx = zmq::Context::new();

        let mut rep_sock = ctx.socket(zmq::REP).unwrap();
        rep_sock.bind("inproc://test_recv_args").unwrap();

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_recv_args").unwrap();

        req_sock.send_str("0", zmq::SNDMORE).unwrap();
        req_sock.send_str("1", 0).unwrap();

        // Eliminate race condition where REP socket prematurely
        // gives up receiving and tries to send err while in
        // incorrect state. This only happens in the test because we
        // call `recv_args()` out of context.
        let _ = rep_sock.recv_string(0).unwrap();

        let result = super::recv_args(&mut rep_sock, 1, 2).unwrap();
        assert_eq!(result, vec!["1"]);
    }

    /// Test providing 1+ args
    #[test]
    fn test_recv_args_ok_variable() {
        let mut ctx = zmq::Context::new();

        let mut rep_sock = ctx.socket(zmq::REP).unwrap();
        rep_sock.bind("inproc://test_recv_args").unwrap();

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_recv_args").unwrap();

        req_sock.send_str("0", zmq::SNDMORE).unwrap();
        req_sock.send_str("1", zmq::SNDMORE).unwrap();
        req_sock.send_str("2", 0).unwrap();

        // Eliminate race condition where REP socket prematurely
        // gives up receiving and tries to send err while in
        // incorrect state. This only happens in the test because we
        // call `recv_args()` out of context.
        let _ = rep_sock.recv_string(0).unwrap();

        let result = super::recv_args(&mut rep_sock, 1, 0).unwrap();
        assert_eq!(result, vec!["1", "2"]);
    }

    /// Test failing less than 2 args
    #[test]
    fn test_recv_args_err_min() {
        let mut ctx = zmq::Context::new();

        let mut rep_sock = ctx.socket(zmq::REP).unwrap();
        rep_sock.bind("inproc://test_recv_args").unwrap();

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_recv_args").unwrap();

        req_sock.send_str("0", zmq::SNDMORE).unwrap();
        req_sock.send_str("1", 0).unwrap();

        // Eliminate race condition where REP socket prematurely
        // gives up receiving and tries to send err while in
        // incorrect state. This only happens in the test because we
        // call `recv_args()` out of context.
        let _ = rep_sock.recv_string(0).unwrap();

        let result = super::recv_args(&mut rep_sock, 2, 2);
        assert!(result.is_err());
    }

    /// Test failing more than 1 arg
    #[test]
    fn test_recv_args_err_max() {
        let mut ctx = zmq::Context::new();

        let mut rep_sock = ctx.socket(zmq::REP).unwrap();
        rep_sock.bind("inproc://test_recv_args").unwrap();

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_recv_args").unwrap();

        req_sock.send_str("0", zmq::SNDMORE).unwrap();
        req_sock.send_str("1", zmq::SNDMORE).unwrap();
        req_sock.send_str("2", 0).unwrap();

        // Eliminate race condition where REP socket prematurely
        // gives up receiving and tries to send err while in
        // incorrect state. This only happens in the test because we
        // call `recv_args()` out of context.
        let _ = rep_sock.recv_string(0).unwrap();

        let result = super::recv_args(&mut rep_sock, 0, 1);
        assert!(result.is_err());
    }

    /// Test sending args
    #[test]
    fn test_send_args() {
        let mut ctx = zmq::Context::new();

        let mut rep_sock = ctx.socket(zmq::REP).unwrap();
        rep_sock.bind("inproc://test_recv_args").unwrap();

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_recv_args").unwrap();

        let req_vec = vec!["moo", "cow"];
        super::send_args(&mut req_sock, req_vec);

        assert_eq!(rep_sock.recv_string(0).unwrap().unwrap(), "moo");
        assert!(rep_sock.get_rcvmore().unwrap());
        assert_eq!(rep_sock.recv_string(0).unwrap().unwrap(), "cow");
    }
}
