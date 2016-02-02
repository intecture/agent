// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

extern crate inagent;
extern crate zmq;

use inagent::{AgentConf, load_agent_conf};
use inagent::file::{File, FileError};
use std::collections::HashMap;
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
    api_sock.connect("inproc:///tmp/inagent.sock").unwrap();

    let mut upload_sock = ctx.socket(zmq::SUB).unwrap();
    upload_sock.set_rcvhwm(TOTAL_SLOTS as i32).unwrap();
    upload_sock.bind(&format!("tcp://*:{}", agent_conf.upload_port)).unwrap();

    let mut download_sock = ctx.socket(zmq::PUB).unwrap();
    download_sock.bind(&format!("tcp://*:{}", agent_conf.download_port)).unwrap();

    let files = Arc::new(RwLock::new(HashMap::new()));
    let files_c = files.clone();

    let chunk_queue: Arc<Mutex<Vec<ChunkQueueItem>>> = Arc::new(Mutex::new(Vec::new()));
    let chunk_queue_c = chunk_queue.clone();

    thread::spawn(move || {
        loop {
            let path = api_sock.recv_string(0).unwrap().unwrap();

            if !api_sock.get_rcvmore().unwrap() {
                continue;
            }

            let hash = api_sock.recv_string(0).unwrap().unwrap().parse::<u64>().unwrap();

            if !api_sock.get_rcvmore().unwrap() {
                continue;
            }

            let size = api_sock.recv_string(0).unwrap().unwrap().parse::<u64>().unwrap();

            if !api_sock.get_rcvmore().unwrap() {
                continue;
            }

            let total_chunks = api_sock.recv_string(0).unwrap().unwrap().parse::<u64>().unwrap();

            if let Ok(f) = File::new(&path, hash, size) {
                for x in 0..total_chunks {
                    chunk_queue_c.lock().unwrap().push(ChunkQueueItem {
                        path: path.clone(),
                        index: x,
                    });
                }

                files_c.write().unwrap().insert(path, f);

                api_sock.send_str("Ok", 0).unwrap();
            } else {
                // XXX Implement Display and Debug traits for error,
                // then send as response.
                api_sock.send_str("Err", 0).unwrap();
            }
        }
    });

    let mut available_slots = TOTAL_SLOTS;

    loop {
        let mut queue = chunk_queue.lock().unwrap();
        let len = if queue.len() < available_slots { queue.len() } else { available_slots };
        for item in queue.drain(0..len) {
            download_sock.send_str(&item.path, zmq::SNDMORE).unwrap();
            download_sock.send_str(&item.index.to_string(), 0).unwrap();
            available_slots -= 1;
        }

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

            let result: Result<(), FileError>;
            let can_retry: bool;
            {
                let mut files_lock = files.write().unwrap();
                let file = files_lock.get_mut(&path).unwrap();
                result = file.write(chunk_index, chunk);
                can_retry = file.can_retry();
            }

            if let Err(_) = result {
                if can_retry {
                    chunk_queue.lock().unwrap().push(ChunkQueueItem {
                        path: path,
                        index: chunk_index,
                    });
                } else {
                    files.write().unwrap().remove(&path);
                    download_sock.send_str(&path, zmq::SNDMORE).unwrap();
                    // XXX Implement Display and Debug traits for
                    // error, then send as response.
                    download_sock.send_str("Failed to upload file!", 0).unwrap();
                }
            }

            available_slots += 1;
        }
    }
}
