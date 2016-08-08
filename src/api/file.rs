// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::Result;
use inapi::{File, Host};
use std::cell::RefCell;
use std::rc::Rc;
use zdaemon::ZMsgExtended;

pub struct FileApi {
    host: Rc<RefCell<Host>>
}

impl FileApi {
    pub fn new(host: Rc<RefCell<Host>>) -> Result<FileApi> {
        Ok(FileApi {
            host: host,
        })
    }

    pub fn is_file(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 1, Some(1), false));
        let msg = try!(ZMsg::new_ok());
        match File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()) {
            Ok(_) => try!(msg.addstr("1")),
            Err(_) => try!(msg.addstr("0")),
        }
        try!(msg.send(sock));
        Ok(())
    }

    pub fn exists(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 1, Some(1), false));
        let file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let exists = try!(file.exists(&mut self.host.borrow_mut()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.addstr(if exists { "1" } else { "0" }));
        try!(msg.send(sock));
        Ok(())
    }

    pub fn delete(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 1, Some(1), false));
        let file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(file.delete(&mut self.host.borrow_mut()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn mv(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 2, Some(2), false));
        let mut file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(file.mv(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn copy(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 2, Some(2), false));
        let file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(file.copy(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn get_owner(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 1, Some(1), false));
        let file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let owner = try!(file.get_owner(&mut self.host.borrow_mut()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send_multi(&sock, &[
            &owner.user_name,
            &owner.user_uid.to_string(),
            &owner.group_name,
            &owner.group_gid.to_string()
        ]));
        Ok(())
    }

    pub fn set_owner(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 3, Some(3), false));
        let file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(file.set_owner(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap(), &request.popstr().unwrap().unwrap()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }

    pub fn get_mode(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 1, Some(1), false));
        let file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        let mode = try!(file.get_mode(&mut self.host.borrow_mut()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.addstr(&mode.to_string()));
        try!(msg.send(sock));
        Ok(())
    }

    pub fn set_mode(&self, sock: &ZSock) -> Result<()> {
        let request = try!(ZMsg::expect_recv(&sock, 2, Some(2), false));
        let file = try!(File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().unwrap()));
        try!(file.set_mode(&mut self.host.borrow_mut(), request.popstr().unwrap().unwrap().parse::<u16>().unwrap()));
        let msg = try!(ZMsg::new_ok());
        try!(msg.send(sock));
        Ok(())
    }
}
