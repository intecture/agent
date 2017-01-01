// Copyright 2015-2017 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq::{ZMsg, ZSock};
use error::{Error, Result};
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

    pub fn is_file(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        match File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?) {
            Ok(_) => msg.addstr("1")?,
            Err(_) => msg.addstr("0")?,
        }
        msg.send(sock)?;
        Ok(())
    }

    pub fn exists(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let exists = file.exists(&mut self.host.borrow_mut())?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.addstr(if exists { "1" } else { "0" })?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn delete(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        file.delete(&mut self.host.borrow_mut())?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn mv(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;
        let mut file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        file.mv(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn copy(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;
        let file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        file.copy(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn get_owner(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let owner = file.get_owner(&mut self.host.borrow_mut())?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
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
        let file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        file.set_owner(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?, &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn get_mode(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 1, Some(1), false)?;
        let file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        let mode = file.get_mode(&mut self.host.borrow_mut())?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.addstr(&mode.to_string())?;
        msg.send(sock)?;
        Ok(())
    }

    pub fn set_mode(&self, sock: &mut ZSock, router_id: &[u8]) -> Result<()> {
        let request = ZMsg::expect_recv(sock, 2, Some(2), false)?;
        let file = File::new(&mut self.host.borrow_mut(), &request.popstr().unwrap().or(Err(Error::MessageUtf8))?)?;
        file.set_mode(&mut self.host.borrow_mut(), request.popstr().unwrap().or(Err(Error::MessageUtf8))?.parse::<u16>().unwrap())?;
        let msg = ZMsg::new_ok()?;
        msg.pushstr("")?;
        msg.pushbytes(router_id)?;
        msg.send(sock)?;
        Ok(())
    }
}
