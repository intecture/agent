// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::Result;
use inapi::{Directory, DirectoryOpts, Host};
use std::cell::RefCell;
use std::rc::Rc;
use zdaemon::ZMsgExtended;

pub struct DirectoryApi {
    host: Rc<RefCell<Host>>,
}

impl DirectoryApi {
    pub fn new(host: Rc<RefCell<Host>>) -> DirectoryApi {
        DirectoryApi {
            host: host,
        }
    }

    pub fn is_directory(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 1, Some(1), false));
        let msg = try!(ZMsg::new_ok());
        match Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()) {
            Ok(_) => try!(msg.addstr("1")),
            Err(_) => try!(msg.addstr("0")),
        }
        try!(msg.send(sock));
        Ok(())
    }

    pub fn exists(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 1, Some(1), false));
        let dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let exists = try!(dir.exists(&mut self.host.borrow_mut()));

        let msg = try!(ZMsg::new_ok());
        try!(msg.addstr(if exists { "1" } else { "0" }));
        try!(msg.send(sock));
        Ok(())
    }

    pub fn create(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 2, Some(2), false));

        let dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));

        let mut opts = vec![];
        if request.popstr().unwrap().unwrap() == "1" {
            opts.push(DirectoryOpts::DoRecursive);
        }

        try!(dir.create(&mut self.host.borrow_mut(), if opts.len() > 0 { Some(opts.as_slice()) } else { None }));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn delete(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 2, Some(2), false));
        let dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));

        let mut opts = vec![];
        if request.popstr().unwrap().unwrap() == "1" {
            opts.push(DirectoryOpts::DoRecursive);
        }

        try!(dir.delete(&mut self.host.borrow_mut(), if opts.len() > 0 { Some(opts.as_slice()) } else { None }));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn mv(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 2, Some(2), false));
        let mut dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(dir.mv(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn get_owner(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 1, Some(1), false));
        let dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let owner = try!(dir.get_owner(&mut self.host.borrow_mut()));

        let msg = try!(ZMsg::new_ok());
        try!(msg.send_multi(sock, &[
            &owner.user_name,
            &owner.user_uid.to_string(),
            &owner.group_name,
            &owner.group_gid.to_string()
        ]));
        Ok(())
    }

    pub fn set_owner(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 3, Some(3), false));
        let dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(dir.set_owner(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap(), &request.popstr().unwrap().unwrap()));

        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn get_mode(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 1, Some(1), false));
        let dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let mode = try!(dir.get_mode(&mut self.host.borrow_mut()));

        let msg = try!(ZMsg::new_ok());
        try!(msg.addstr(&mode.to_string()));
        try!(msg.send(sock));
        Ok(())
    }

    pub fn set_mode(&self, sock: &mut ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(sock, 2, Some(2), false));
        let dir = try!(Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(dir.set_mode(&mut self.host.borrow_mut(), request.popstr().unwrap().unwrap().parse::<u16>().unwrap()));

        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }
}
