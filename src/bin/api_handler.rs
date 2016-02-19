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

use inagent::{AgentConf, load_agent_conf, recv_args, send_args};
use inapi::{Command, File, Host, ProviderFactory, Telemetry};
use rustc_serialize::json;
use std::error::Error;
use std::process::exit;

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
    file_sock.bind("ipc:///tmp/inagent.sock").unwrap();

    let mut host = Host::new();

    loop {
        let endpoint_msg = api_sock.recv_msg(0).unwrap();
        let endpoint = endpoint_msg.as_str().unwrap();

        match endpoint {
            "command::exec" => {
                let args;

                match recv_args(&mut api_sock, 1, Some(1), false) {
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

                match recv_args(&mut api_sock, 1, Some(1), false) {
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

                match recv_args(&mut api_sock, 1, Some(1), false) {
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
            "file::delete" => {
                let args;

                match recv_args(&mut api_sock, 1, Some(1), false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.delete(&mut host) {
                            Ok(_) => send_args(&mut api_sock, vec!["Ok"]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::mv" => {
                let args;

                match recv_args(&mut api_sock, 2, Some(2), false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.mv(&mut host, &args[1]) {
                            Ok(_) => send_args(&mut api_sock, vec!["Ok"]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::copy" => {
                let args;

                match recv_args(&mut api_sock, 2, Some(2), false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.copy(&mut host, &args[1]) {
                            Ok(_) => send_args(&mut api_sock, vec!["Ok"]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::get_owner" => {
                let args;

                match recv_args(&mut api_sock, 1, Some(1), false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.get_owner(&mut host) {
                            Ok(owner) => send_args(&mut api_sock, vec!["Ok", &owner.user_name, &owner.user_uid.to_string(), &owner.group_name, &owner.group_gid.to_string()]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::set_owner" => {
                let args;

                match recv_args(&mut api_sock, 3, Some(3), false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.set_owner(&mut host, &args[1], &args[2]) {
                            Ok(_) => send_args(&mut api_sock, vec!["Ok"]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::get_mode" => {
                let args;

                match recv_args(&mut api_sock, 1, Some(1), false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.get_mode(&mut host) {
                            Ok(mode) => send_args(&mut api_sock, vec!["Ok", &mode.to_string()]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::set_mode" => {
                let args;

                match recv_args(&mut api_sock, 2, Some(2), false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match File::new(&mut host, &args[0]) {
                    Ok(file) => {
                        match file.set_mode(&mut host, args[1].parse::<u16>().unwrap()) {
                            Ok(_) => send_args(&mut api_sock, vec!["Ok"]),
                            Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                        }
                    },
                    Err(e) => send_args(&mut api_sock, vec!["Err", e.description()]),
                }
            },
            "file::upload" => {
                let args;

                match recv_args(&mut api_sock, 4, None, false) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                send_args(&mut file_sock, args.iter().map(|a| a.as_ref()).collect());

                match recv_args(&mut file_sock, 1, Some(2), true) {
                    Ok(r) => match r.first().unwrap().as_ref() {
                        "Ok" | "Err" => send_args(&mut api_sock, r.iter().map(|a| a.as_ref()).collect()),
                        _ => send_args(&mut api_sock, vec![ "Err", "Unexpected response" ]),
                    },
                    Err(_) => send_args(&mut api_sock, vec!["Err", "Could not recv from file handler"]),
                }
            },
            "package::default_provider" => {
                if recv_args(&mut api_sock, 0, None, false).is_err() {
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
                if recv_args(&mut api_sock, 0, None, false).is_err() {
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
                // recv() any errant frames before trying to send
                recv_args(&mut api_sock, 0, None, false).unwrap();

                api_sock.send_str("Err", zmq::SNDMORE).unwrap();
                api_sock.send_str(&format!("Invalid endpoint {}", endpoint), 0).unwrap();
                continue;
            }
        }
    }
}
