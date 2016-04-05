// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::{Error, Result};
use std::error::Error as StdError;
use std::fmt::{Debug, Display};

pub struct Msg;

impl Msg {
    /// Receive a variable number of args from sock. We expect the number
    /// of args from `sock` to be between `min` and `max`.
    /// If max = None then we allow a variable number of args.
    pub fn expect_recv(sock: &ZSock, min: usize, max: Option<usize>, block: bool) -> Result<ZMsg> {
        // Avoid blocking unless it is explicitly required.
        let zmsg = if sock.rcvmore() || block {
            try!(ZMsg::recv(sock))
        } else {
            ZMsg::new()
        };

        if min > zmsg.size() || (max.is_some() && max.unwrap() < zmsg.size()) {
            if try!(Self::sock_is_writeable(sock)) {
                let zmsg = ZMsg::new();
                try!(zmsg.addstr("Err"));
                try!(zmsg.addstr("Invalid args count"));
                try!(zmsg.send(sock));
            }

            Err(Error::InvalidArgsCount)
        } else {
            Ok(zmsg)
        }
    }

    pub fn send(sock: &ZSock, frames: Vec<&str>) -> Result<()> {
        let zmsg = ZMsg::new();
        for f in frames {
            try!(zmsg.addstr(&f));
        }
        try!(zmsg.send(sock));
        Ok(())
    }

    pub fn send_ok(sock: &ZSock, mut frames: Vec<&str>) -> Result<()> {
        frames.insert(0, "Ok");
        Self::send(sock, frames)
    }

    pub fn send_err(sock: &ZSock, err: Error) -> Result<()> {
        Self::send(sock, vec!["Err", err.description()])
    }

    fn sock_is_writeable(sock: &ZSock) -> Result<bool> {
        match try!(sock.type_str()).unwrap() {
            "PAIR" |
            "PUB" |
            "REQ" |
            "REP" |
            "DEALER" |
            "ROUTER" |
            "PUSH" |
            "XSUB" => Ok(true),

            "SUB" |
            "PULL" |
            "XPUB" => Ok(false),

            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use czmq::{ZMsg, ZSock};
    use super::*;

    /// Test providing 1 and only 1 arg
    #[test]
    fn test_recv_args_ok_eq() {
        let mut server = ZSock::new_rep("inproc://msg_recv_args_ok_eq").unwrap();
        let mut client = ZSock::new_req("inproc://msg_recv_args_ok_eq").unwrap();

        let msg = ZMsg::new();
        msg.addstr("0").unwrap();
        msg.send(&client);

        let rcv_msg = Msg::expect_recv(&server, 1, Some(1), true).unwrap();
        assert_eq!(rcv_msg.popstr().unwrap().unwrap(), "0");
    }

    /// Test providing 1 or more args
    #[test]
    fn test_recv_args_ok_range() {
        let mut server = ZSock::new_rep("inproc://msg_recv_args_ok_range").unwrap();
        let mut client = ZSock::new_req("inproc://msg_recv_args_ok_range").unwrap();

        let msg = ZMsg::new();
        msg.addstr("0").unwrap();
        msg.send(&client);

        let rcv_msg = Msg::expect_recv(&server, 1, Some(2), true).unwrap();
        assert_eq!(rcv_msg.popstr().unwrap().unwrap(), "0");
    }

    /// Test providing 1+ args
    #[test]
    fn test_recv_args_ok_variable() {
        let mut server = ZSock::new_rep("inproc://msg_recv_args_ok_variable").unwrap();
        let mut client = ZSock::new_req("inproc://msg_recv_args_ok_variable").unwrap();

        let msg = ZMsg::new();
        msg.addstr("0").unwrap();
        msg.addstr("1").unwrap();
        msg.addstr("2").unwrap();
        msg.send(&client);

        let rcv_msg = Msg::expect_recv(&server, 2, None, true).unwrap();
        assert_eq!(rcv_msg.popstr().unwrap().unwrap(), "0");
        assert_eq!(rcv_msg.popstr().unwrap().unwrap(), "1");
        assert_eq!(rcv_msg.popstr().unwrap().unwrap(), "2");
    }

    /// Test failing less than 3 args
    #[test]
    fn test_recv_args_err_min() {
        let mut server = ZSock::new_rep("inproc://msg_recv_args_err_min").unwrap();
        let mut client = ZSock::new_req("inproc://msg_recv_args_err_min").unwrap();

        let msg = ZMsg::new();
        msg.addstr("0").unwrap();
        msg.addstr("1").unwrap();
        msg.send(&client);

        let rcv_msg = Msg::expect_recv(&server, 3, None, true);
        assert!(rcv_msg.is_err());
    }

    /// Test failing more than 1 arg
    #[test]
    fn test_recv_args_err_max() {
        let mut server = ZSock::new_rep("inproc://msg_recv_args_err_max").unwrap();
        let mut client = ZSock::new_req("inproc://msg_recv_args_err_max").unwrap();

        let msg = ZMsg::new();
        msg.addstr("0").unwrap();
        msg.addstr("1").unwrap();
        msg.addstr("2").unwrap();
        msg.send(&client);

        let rcv_msg = Msg::expect_recv(&server, 0, Some(1), true);
        assert!(rcv_msg.is_err());
    }
}
