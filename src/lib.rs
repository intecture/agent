// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate rustc_serialize;
extern crate zmq;

mod config;
mod error;
pub mod file;

pub use config::agent::AgentConf;
pub use config::Config;
pub use error::{Error, Result};
use std::path::PathBuf;

pub fn load_agent_conf<'a>() -> Result<AgentConf> {
    for p in ["/usr/local/etc", "/etc"].iter() {
        let mut path = PathBuf::from(p);
        path.push("intecture");
        path.push("agent.json");

        match AgentConf::load(&path) {
            Ok(conf) => return Ok(conf),
            Err(_) => continue,
        }
    }

    Err(Error::MissingConf)
}

/// Receive a variable number of args from sock. We expect the number
/// of args from `sock` to be between `min` and `max`.
/// If max = 0 then we allow a variable number of args.
pub fn recv_args(sock: &mut zmq::Socket, min: u8, max: Option<u8>, b: bool) -> Result<Vec<String>> {
    let mut args: Vec<String> = vec![];
    let mut block = b;
    let mut counter = 0;

    // Always receive all args, otherwise our socket will be in the
    // wrong state to send messages.
    while sock.get_rcvmore().unwrap() || block {
        let s = sock.recv_string(0).unwrap().unwrap();
        args.push(s);
        counter += 1;
        block = false;
    }

    if min > counter || (max.is_some() && max.unwrap() < counter) {
        if sock_is_writeable(sock.get_socket_type().unwrap()) {
            sock.send_str("Err", zmq::SNDMORE).unwrap();
            sock.send_str("Invalid args count", 0).unwrap();
        }

        return Err(Error::InvalidArgsCount);
    }

    Ok(args)
}

pub fn send_args(sock: &mut zmq::Socket, args: Vec<&str>) {
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

fn sock_is_writeable(sock_type: zmq::SocketType) -> bool {
    match sock_type {
        zmq::SocketType::PAIR |
        zmq::SocketType::PUB |
        zmq::SocketType::REQ |
        zmq::SocketType::REP |
        zmq::SocketType::DEALER |
        zmq::SocketType::ROUTER |
        zmq::SocketType::PUSH |
        zmq::SocketType::XSUB => true,

        zmq::SocketType::SUB |
        zmq::SocketType::PULL |
        zmq::SocketType::XPUB => false
    }
}

#[cfg(test)]
mod tests {
    use super::{recv_args, send_args};
    use zmq;

    // XXX Need to mock FS before we can effectively test this
    // #[test]
    // fn test_load_agent_conf() {
    //
    // }

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

        let result = recv_args(&mut rep_sock, 2, Some(2), true);
        assert_eq!(result.unwrap(), vec!["0", "1"]);
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

        let result = recv_args(&mut rep_sock, 2, Some(2), true).unwrap();
        assert_eq!(result, vec!["0", "1"]);
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

        let result = recv_args(&mut rep_sock, 1, None, true).unwrap();
        assert_eq!(result, vec!["0", "1", "2"]);
    }

    /// Test failing less than 3 args
    #[test]
    fn test_recv_args_err_min() {
        let mut ctx = zmq::Context::new();

        let mut rep_sock = ctx.socket(zmq::REP).unwrap();
        rep_sock.bind("inproc://test_recv_args").unwrap();

        let mut req_sock = ctx.socket(zmq::REQ).unwrap();
        req_sock.connect("inproc://test_recv_args").unwrap();

        req_sock.send_str("0", zmq::SNDMORE).unwrap();
        req_sock.send_str("1", 0).unwrap();

        let result = recv_args(&mut rep_sock, 3, Some(3), true);
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

        let result = recv_args(&mut rep_sock, 0, Some(1), true);
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
        send_args(&mut req_sock, req_vec);

        assert_eq!(rep_sock.recv_string(0).unwrap().unwrap(), "moo");
        assert!(rep_sock.get_rcvmore().unwrap());
        assert_eq!(rep_sock.recv_string(0).unwrap().unwrap(), "cow");
    }
}
