// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::{Error, Result};
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

    pub fn is_directory(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let msg = ZMsg::new_ok(Some(router_id))?;
        match Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?) {
            Ok(_) => msg.addstr("1")?,
            Err(_) => msg.addstr("0")?,
        }
        msg.send(sock)?;
        Ok(())
    }

    pub fn exists(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let exists = dir.exists(&mut self.host.borrow_mut())?;

        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.addstr(if exists { "1" } else { "0" })?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn create(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;

        let dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;

        let mut opts = vec![];
        if request.popstr().unwrap().or(Err(Error::MessageUtf8))? == "1" {
            opts.push(DirectoryOpts::DoRecursive);
        }

        dir.create(&mut self.host.borrow_mut(), if opts.len() > 0 { Some(opts.as_slice()) } else { None })?;
        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn delete(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;
        let dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;

        let mut opts = vec![];
        if request.popstr().unwrap().or(Err(Error::MessageUtf8))? == "1" {
            opts.push(DirectoryOpts::DoRecursive);
        }

        dir.delete(&mut self.host.borrow_mut(), if opts.len() > 0 { Some(opts.as_slice()) } else { None })?;
        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn mv(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;
        let mut dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        dir.mv(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn get_owner(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let owner = dir.get_owner(&mut self.host.borrow_mut())?;

        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.send_multi(sock, &[
            &owner.user_name,
            &owner.user_uid.to_string(),
            &owner.group_name,
            &owner.group_gid.to_string()
        ])?;
        Ok(())
    }

    pub fn set_owner(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 3, Some(3), false)?;
        let dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        dir.set_owner(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?, &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;

        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn get_mode(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let mode = dir.get_mode(&mut self.host.borrow_mut())?;

        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.addstr(&mode.to_string())?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn set_mode(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;
        let dir = Directory::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        dir.set_mode(&mut self.host.borrow_mut(), request.popstr().unwrap().or(Err(Error::MessageUtf8))?.parse::<u16>().unwrap())?;

        let msg = ZMsg::new_ok(Some(router_id))?;
        msg.send(sock)?;
        Ok(())
    }
}
