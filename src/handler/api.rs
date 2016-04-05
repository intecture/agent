// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use config::agent::AgentConf;
use czmq::{ZCert, ZMsg, ZSock};
use error::{Error, Result};
use inapi::{Command, Directory, DirectoryOpts, File, Host, ProviderFactory, Service, ServiceRunnable, Telemetry};
use msg::Msg;
use rustc_serialize::json;
use std::error::Error as StdError;
use std::process::exit;
use super::Handler;

pub struct ApiHandler<'a> {
    conf: &'a AgentConf,
    cert: &'a ZCert,
}

impl<'a> Handler<'a, ApiHandler<'a>> for ApiHandler<'a> {
    fn new(conf: &'a AgentConf, cert: &'a ZCert) -> ApiHandler<'a> {
        ApiHandler {
            conf: conf,
            cert: cert,
        }
    }

    fn run(&self) -> Result<()> {
        let mut host = Host::new();

        let api_sock = try!(ZSock::new_rep(&format!("tcp://*:{}", self.conf.api_port)));
        api_sock.set_zap_domain("intecture");
        api_sock.set_curve_server(true);
        self.cert.apply(&api_sock);

        let file_sock = try!(ZSock::new_pair("inproc://api_file_link"));

        loop {
            let endpoint: String;

            match api_sock.recv_str() {
                Ok(result) => match result {
                    Ok(msg) => endpoint = msg,
                    Err(_) => continue,
                },
                Err(_) => continue,
            }

            let result = match endpoint.as_ref() {
                "command::exec" => command_exec(&api_sock, &mut host),
                "directory::is_directory" => directory_is_directory(&api_sock, &mut host),
                "directory::exists" => directory_exists(&api_sock, &mut host),
                "directory::create" => directory_create(&api_sock, &mut host),
                "directory::delete" => directory_delete(&api_sock, &mut host),
                "directory::mv" => directory_mv(&api_sock, &mut host),
                "directory::get_owner" => directory_get_owner(&api_sock, &mut host),
                "directory::set_owner" => directory_set_owner(&api_sock, &mut host),
                "directory::get_mode" => directory_get_mode(&api_sock, &mut host),
                "directory::set_mode" => directory_set_mode(&api_sock, &mut host),
                "file::is_file" => file_is_file(&api_sock, &mut host),
                "file::exists" => file_exists(&api_sock, &mut host),
                "file::delete" => file_delete(&api_sock, &mut host),
                "file::mv" => file_mv(&api_sock, &mut host),
                "file::copy" => file_copy(&api_sock, &mut host),
                "file::get_owner" => file_get_owner(&api_sock, &mut host),
                "file::set_owner" => file_set_owner(&api_sock, &mut host),
                "file::get_mode" => file_get_mode(&api_sock, &mut host),
                "file::set_mode" => file_set_mode(&api_sock, &mut host),
                "file::upload" => file_upload(&api_sock, &file_sock, &mut host),
                "package::default_provider" => package_default_provider(&api_sock, &mut host),
                "service::action" => service_action(&api_sock, &mut host),
                "telemetry" => telemetry(&api_sock, &mut host),
                _ => {
                    // recv() any errant frames before trying to send
                    Msg::expect_recv(&api_sock, 0, None, false).unwrap();
                    Err(Error::InvalidEndpoint)
                }
            };

            match result {
                Ok(frames) => try!(Msg::send_ok(&api_sock, frames)),
                Err(e) => try!(Msg::send_err(&api_sock, e)),
            }
        }

        Ok(())
    }
}

fn command_exec<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    let cmd = Command::new(&try!(request.popstr()).unwrap());
    let result = try!(cmd.exec(host));

    Ok(vec![
        &result.exit_code.to_string(),
        &result.stdout,
        &result.stderr
    ])
}

fn directory_is_directory<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    match Directory::new(host, &try!(request.popstr()).unwrap()) {
        Ok(_) => Ok(vec!["1"]),
        Err(_) => Ok(vec!["0"]),
    }
}

fn directory_exists<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    let dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));
    let exists = try!(dir.exists(host));
    Ok(vec![if exists { "1" } else { "0" }])
}

fn directory_create<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));

    let mut opts = vec![];
    if try!(request.popstr()).unwrap() == "1" {
        opts.push(DirectoryOpts::DoRecursive);
    }

    dir.create(host, if opts.len() > 0 { Some(opts.as_slice()) } else { None });
    Ok(vec![])
}

fn directory_delete<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));

    let mut opts = vec![];
    if try!(request.popstr()).unwrap() == "1" {
        opts.push(DirectoryOpts::DoRecursive);
    }

    try!(dir.delete(host, if opts.len() > 0 { Some(opts.as_slice()) } else { None }));
    Ok(vec![])
}

fn directory_mv<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let mut dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));
    try!(dir.mv(host, &try!(request.popstr()).unwrap()));
    Ok(vec![])
}

fn directory_get_owner<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    let dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));
    let owner = try!(dir.get_owner(host));
    Ok(vec![
        &owner.user_name,
        &owner.user_uid.to_string(),
        &owner.group_name,
        &owner.group_gid.to_string()
    ])
}

fn directory_set_owner<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 3, Some(3), false));
    let dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));
    try!(dir.set_owner(host, &try!(request.popstr()).unwrap(), &try!(request.popstr()).unwrap()));
    Ok(vec![])
}

fn directory_get_mode<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    let dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));
    let mode = try!(dir.get_mode(host));
    Ok(vec![&mode.to_string()])
}

fn directory_set_mode<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let dir = try!(Directory::new(host, &try!(request.popstr()).unwrap()));
    try!(dir.set_mode(host, try!(request.popstr()).unwrap().parse::<u16>().unwrap()));
    Ok(vec![])
}

fn file_is_file<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    match File::new(host, &try!(request.popstr()).unwrap()) {
        Ok(_) => Ok(vec!["1"]),
        Err(_) => Ok(vec!["0"]),
    }
}

fn file_exists<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    let file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    let exists = try!(file.exists(host));
    Ok(vec![if exists { "1" } else { "0" }])
}

fn file_delete<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    try!(file.delete(host));
    Ok(vec![])
}

fn file_mv<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let mut file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    try!(file.mv(host, &try!(request.popstr()).unwrap()));
    Ok(vec![])
}

fn file_copy<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    try!(file.copy(host, &try!(request.popstr()).unwrap()));
    Ok(vec![])
}

fn file_get_owner<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    let file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    let owner = try!(file.get_owner(host));
    Ok(vec![
        &owner.user_name,
        &owner.user_uid.to_string(),
        &owner.group_name,
        &owner.group_gid.to_string()
    ])
}

fn file_set_owner<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 3, Some(3), false));
    let file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    try!(file.set_owner(host, &try!(request.popstr()).unwrap(), &try!(request.popstr()).unwrap()));
    Ok(vec![])
}

fn file_get_mode<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 1, Some(1), false));
    let file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    let mode = try!(file.get_mode(host));
    Ok(vec![&mode.to_string()])
}

fn file_set_mode<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let file = try!(File::new(host, &try!(request.popstr()).unwrap()));
    try!(file.set_mode(host, try!(request.popstr()).unwrap().parse::<u16>().unwrap()));
    Ok(vec![])
}

fn file_upload<'a>(sock: &ZSock, file_sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 4, None, false));
    try!(request.send(file_sock));

    let request = try!(Msg::expect_recv(&sock, 1, Some(2), true));
    match try!(request.popstr()).unwrap().as_ref() {
        "Ok" => Ok(vec![]),
        "Err" => Err(Error::FileError(try!(request.popstr()).unwrap())),
        _ => Err(Error::InvalidStatus),
    }
}

fn package_default_provider<'a>(sock: &ZSock, host: &'a mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 0, None, false));
    let provider = try!(ProviderFactory::create(host, None));
    Ok(vec![&provider.get_providers().to_string()])
}

fn service_action<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 2, Some(2), false));
    let runnable = try!(request.popstr()).unwrap();
    let service = Service::new_service(ServiceRunnable::Service(&runnable), None);
    let result = try!(service.action(host, &try!(request.popstr()).unwrap()));
    Ok(vec![
        &result.exit_code.to_string(),
        &result.stdout,
        &result.stderr,
    ])
}

fn telemetry<'a>(sock: &ZSock, host: &mut Host) -> Result<Vec<&'a str>> {
    let request = try!(Msg::expect_recv(&sock, 0, None, false));
    let telemetry = try!(Telemetry::init(host));
    let json = try!(json::encode(&telemetry));
    Ok(vec![&json])
}
