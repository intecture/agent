extern crate zmq;
extern crate rustc_serialize;

mod command;
mod config;

use config::agent::AgentConf;
use config::Config;
use std::error::Error;
use std::path::PathBuf;
use std::process::exit;

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let mut ctx = zmq::Context::new();
    run(&mut ctx);
}

fn run(ctx: &mut zmq::Context) {
    // Load agent config
    let agent_conf: AgentConf;
    match load_agent_conf() {
        Ok(conf) => agent_conf = conf,
        Err(e) => {
            print_err(&e);
            exit(1);
        },
    }

    let mut listen_sock = ctx.socket(zmq::REP).unwrap();
    let dsn = format!("tcp://*:{}", agent_conf.listen_port);
    listen_sock.bind(&dsn).unwrap();

    loop {
        let endpoint_msg = listen_sock.recv_msg(0).unwrap();
        let endpoint = endpoint_msg.as_str().unwrap();

        match endpoint {
            "command::exec" => {
                let args;

                match recv_args(&mut listen_sock, 1, 1) {
                    Ok(r) => args = r,
                    Err(_) => continue,
                }

                match command::exec(&args[0]) {
                    Ok(output) => send_args(&mut listen_sock, vec![
                        "Ok",
                        &output.status.code().unwrap().to_string(),
                        &String::from_utf8(output.stdout).unwrap(),
                        &String::from_utf8(output.stderr).unwrap()
                    ]),
                    Err(e) => send_args(&mut listen_sock, vec!["Err", e.description()]),
                }
            },
            _ => {
                listen_sock.send_str("Err", zmq::SNDMORE).unwrap();
                listen_sock.send_str(&format!("Invalid endpoint {}", endpoint), 0).unwrap();
                continue;
            }
        }
    }
}

/// Receive a variable number of args from sock. We expect the number
/// of args from sock to match num.
/// If num = 0 then we allow a variable number of args.
fn recv_args(sock: &mut zmq::Socket, min: u8, max: u8) -> Result<Vec<String>, AgentError> {
    let mut args: Vec<String> = vec![];
    let mut counter = 0;

    // Always receive all args, otherwise our socket will be in the
    // wrong state to send messages.
    while sock.get_rcvmore().unwrap() == true {
        match sock.recv_string(0) {
            Ok(str) => {
                args.push(str.unwrap());
                counter += 1;
            },
            Err(e) => return Err(AgentError {
                message: "Could not receive from socket",
                root: RootError::ZmqError(e),
            }),
        }
    }

    if min > counter || (max < counter && max > 0) {
        sock.send_str("Err", zmq::SNDMORE).unwrap();
        sock.send_str("Missing argument", 0).unwrap();
        return Err(AgentError {
            message: "Received more args than expected",
            root: RootError::None(()),
        });
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

fn load_agent_conf<'a>() -> Result<AgentConf, AgentError<'a>> {
    for p in ["/usr/local/etc", "/etc"].iter() {
        let mut path = PathBuf::from(p);
        path.push("intecture");
        path.push("agent.json");

        match AgentConf::load(&path) {
            Ok(conf) => return Ok(conf),
            Err(_) => continue,
        }
    }

    Err(AgentError {
        message: "Could not load agent.json",
        root: RootError::None(()),
    })
}

#[derive(Debug)]
pub struct AgentError<'a> {
    message: &'a str,
    root: RootError,
}

#[derive(Debug)]
pub enum RootError {
    None(()),
    ZmqError(zmq::Error),
}

fn print_err(e: &AgentError) {
    println!("{}", e.message);
    println!("{:?}", e.root);
}