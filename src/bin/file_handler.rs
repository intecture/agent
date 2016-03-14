// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate inagent;
extern crate zmq;

use inagent::{AgentConf, load_agent_conf, recv_args, Result, send_args};
use inagent::file::{File, convert_opt_args};
use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;

const TOTAL_SLOTS: usize = 20;

struct ChunkQueueItem {
    path: String,
    index: u64,
}

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

    let mut api_sock = ctx.socket(zmq::PAIR).unwrap();
    api_sock.connect("ipc:///tmp/inagent.sock").unwrap();

    let mut queue_sock = ctx.socket(zmq::PULL).unwrap();
    queue_sock.bind("inproc://slice_queue").unwrap();

    let mut queue_api_sock = ctx.socket(zmq::PUSH).unwrap();
    queue_api_sock.connect("inproc://slice_queue").unwrap();

    let mut queue_file_sock = ctx.socket(zmq::PUSH).unwrap();
    queue_file_sock.connect("inproc://slice_queue").unwrap();

    let mut upload_sock = ctx.socket(zmq::SUB).unwrap();
    upload_sock.set_rcvhwm(TOTAL_SLOTS as i32).unwrap();
    upload_sock.bind(&format!("tcp://*:{}", agent_conf.upload_port)).unwrap();
    upload_sock.set_subscribe("".as_bytes()).unwrap();

    let mut download_sock = ctx.socket(zmq::PUB).unwrap();
    download_sock.bind(&format!("tcp://*:{}", agent_conf.download_port)).unwrap();

    let files = Arc::new(RwLock::new(HashMap::new()));
    let files_c = files.clone();

    thread::spawn(move || {
        loop {
            let mut args;

            match recv_args(&mut api_sock, 4, None, true) {
                Ok(r) => args = r,
                Err(_) => continue,
            }

            let path = args.remove(0).to_string();
            let hash = args.remove(0).parse::<u64>().unwrap();
            let size = args.remove(0).parse::<u64>().unwrap();
            let total_chunks = args.remove(0).parse::<u64>().unwrap();

            match File::new(&path, hash, size, total_chunks, convert_opt_args(args)) {
                Ok(f) => {
                    for x in 0..total_chunks {
                        send_args(&mut queue_api_sock, vec!["QUEUE", &path, &x.to_string()]);
                    }

                    files_c.write().unwrap().insert(path, f);

                    api_sock.send_str("Ok", 0).unwrap();
                },
                Err(e) => {
                    api_sock.send_str("Err", zmq::SNDMORE).unwrap();
                    api_sock.send_str(e.description(), 0).unwrap();
                }
            }
        }
    });

    let mut available_slots = TOTAL_SLOTS;
    let chunk_queue: Arc<Mutex<Vec<ChunkQueueItem>>> = Arc::new(Mutex::new(Vec::new()));

    thread::spawn(move || {
        loop {
            let args;

            let cmd = queue_sock.recv_string(0).unwrap().unwrap();

            match cmd.as_ref() {
                "READY" | "QUEUE" => {
                    let mut queue = chunk_queue.lock().unwrap();

                    if cmd == "QUEUE" {
                        match recv_args(&mut queue_sock, 2, Some(2), false) {
                            Ok(r) => args = r,
                            Err(_) => continue,
                        }

                        queue.push(ChunkQueueItem {
                            path: args[0].to_string(),
                            index: args[1].parse::<u64>().unwrap(),
                        });
                    } else {
                        available_slots += 1
                    }

                    if available_slots > 0 && queue.len() > 0 {
                        let item = queue.remove(0);
                        download_sock.send_str(&item.path, zmq::SNDMORE).unwrap();
                        download_sock.send_str("Chk", zmq::SNDMORE).unwrap();
                        download_sock.send_str(&item.index.to_string(), 0).unwrap();
                        available_slots -= 1;
                    }
                },
                "DONE" => {
                    match recv_args(&mut queue_sock, 1, Some(1), false) {
                        Ok(r) => args = r,
                        Err(_) => continue,
                    }

                    download_sock.send_str(&args[0], zmq::SNDMORE).unwrap();
                    download_sock.send_str("Done", 0).unwrap();
                },
                "ERR" => {
                    match recv_args(&mut queue_sock, 2, Some(2), false) {
                        Ok(r) => args = r,
                        Err(_) => continue,
                    }

                    download_sock.send_str(&args[0], zmq::SNDMORE).unwrap();
                    download_sock.send_str("Err", zmq::SNDMORE).unwrap();
                    download_sock.send_str(&args[1], 0).unwrap();
                },
                _ => unimplemented!(),
            }
        }
    });

    loop {
        let path = upload_sock.recv_string(0).unwrap().unwrap();

        if !upload_sock.get_rcvmore().unwrap() {
            continue;
        }

        if files.read().unwrap().contains_key(&path) {
            let chunk_index = upload_sock.recv_string(0).unwrap().unwrap().parse::<u64>().unwrap();

            if !upload_sock.get_rcvmore().unwrap() {
                continue;
            }

            let chunk = upload_sock.recv_bytes(0).unwrap();

            let result: Result<()>;
            let is_finished: bool;
            let can_retry: bool;
            // Artificially scope reference to files_lock to ensure
            // it releases early.
            {
                let mut files_lock = files.write().unwrap();
                let file = files_lock.get_mut(&path).unwrap();

                result = file.write(chunk_index, chunk);
                is_finished = file.is_finished();
                can_retry = file.can_retry();
            }

            match result {
                Ok(_) => {
                    if is_finished {
                        let mut files_lock = files.write().unwrap();
                        match files_lock.get_mut(&path).unwrap().install() {
                            Ok(()) => {
                                files_lock.remove(&path);
                                send_args(&mut queue_file_sock, vec!["DONE", &path]);
                            },
                            Err(e) => send_args(&mut queue_file_sock, vec!["ERR", &path, e.description()]),
                        }
                    }
                },
                Err(e) => {
                    if can_retry {
                        send_args(&mut queue_file_sock, vec!["QUEUE", &path, &chunk_index.to_string()]);
                    } else {
                        files.write().unwrap().remove(&path);
                        send_args(&mut queue_file_sock, vec!["ERR", &path, e.description()]);
                    }
                },
            }

            send_args(&mut queue_file_sock, vec!["READY"]);
        }
    }
}
