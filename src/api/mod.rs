// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

mod command;
mod directory;
mod file;
mod package;
mod service;
mod telemetry;

use czmq::{ZCert, ZFrame, ZMsg, ZSock, SocketType};
use error::Result;
use inapi::Host;
use self::command::CommandApi;
use self::directory::DirectoryApi;
use self::file::FileApi;
use self::package::PackageApi;
use self::service::ServiceApi;
use self::telemetry::TelemetryApi;
use std::cell::RefCell;
use std::rc::Rc;
use std::result::Result as StdResult;
use zdaemon::{Api, Error as DError, ZMsgExtended};

pub fn endpoint(api_port: u32, cert: &ZCert) -> Result<Api> {
    let mut api_sock = ZSock::new(SocketType::ROUTER);
    cert.apply(&mut api_sock);
    api_sock.set_zap_domain("agent.intecture");
    api_sock.set_curve_server(true);
    api_sock.set_linger(1000);
    api_sock.bind(&format!("tcp://*:{}", api_port))?;

    let mut api = Api::new(api_sock);

    let path: Option<String> = None;
    let host = Rc::new(RefCell::new(Host::local(path)?));

    let host_clone = host.clone();
    api.add("command::exec", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = CommandApi::exec(sock, &mut host_clone.borrow_mut(), &i); error_handler(sock, r, &i) });

    let directory_api = Rc::new(DirectoryApi::new(host.clone()));
    let directory_clone = directory_api.clone();
    api.add("directory::is_directory", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.is_directory(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::exists", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.exists(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::create", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.create(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::delete", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.delete(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::mv", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.mv(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::get_owner", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.get_owner(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::set_owner", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.set_owner(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::get_mode", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.get_mode(sock, &i); error_handler(sock, r, &i) });
    let directory_clone = directory_api.clone();
    api.add("directory::set_mode", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = directory_clone.set_mode(sock, &i); error_handler(sock, r, &i) });

    let file_api = Rc::new(FileApi::new(host.clone())?);
    let file_clone = file_api.clone();
    api.add("file::is_file", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.is_file(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::exists", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.exists(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::delete", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.delete(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::mv", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.mv(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::copy", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.copy(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::get_owner", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.get_owner(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::set_owner", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.set_owner(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::get_mode", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.get_mode(sock, &i); error_handler(sock, r, &i) });
    let file_clone = file_api.clone();
    api.add("file::set_mode", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = file_clone.set_mode(sock, &i); error_handler(sock, r, &i) });

    let host_clone = host.clone();
    api.add("package::default_provider", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = PackageApi::default_provider(sock, &mut host_clone.borrow_mut(), &i); error_handler(sock, r, &i) });

    let host_clone = host.clone();
    api.add("service::action", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = ServiceApi::action(sock, &mut host_clone.borrow_mut(), &i); error_handler(sock, r, &i) });

    let host_clone = host.clone();
    api.add("telemetry", move |sock: &mut ZSock, _: ZFrame, id: Option<Vec<u8>>| { let i = id.unwrap(); let r = TelemetryApi::get(sock, &mut host_clone.borrow_mut(), &i); error_handler(sock, r, &i) });

    Ok(api)
}

fn error_handler(sock: &mut ZSock, result: Result<()>, router_id: &[u8]) -> StdResult<(), DError> {
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            let derror: DError = e.into();
            let msg = ZMsg::new_err(&derror, Some(router_id))?;
            msg.send(sock)?;
            Err(derror)
        }
    }
}

#[cfg(test)]
mod tests {
    use czmq::{ZMsg, ZSock};
    use error::Error;
    use std::error::Error as StdError;
    use super::error_handler;

    #[test]
    fn test_error_handler() {
        let mut client = ZSock::new_push("inproc://server_test_error_handler").unwrap();
        let mut server = ZSock::new_pull("inproc://server_test_error_handler").unwrap();
        server.set_rcvtimeo(Some(500));

        let e = server.send_str("fail").unwrap_err();
        let e_desc = e.description().to_string();
        assert!(error_handler(&mut client, Err(Error::Czmq(e)), b"router_id").is_err());

        let msg = ZMsg::recv(&mut server).unwrap();
        assert_eq!(msg.popstr().unwrap().unwrap(), "router_id");
        assert_eq!(msg.popstr().unwrap().unwrap(), "Err");
        assert_eq!(msg.popstr().unwrap().unwrap(), e_desc);
    }
}
