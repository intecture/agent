// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use config::agent::AgentConf;
use czmq::{ZCert, ZSock};
use error::Result;
use file::{File, convert_opt_args};
use msg::Msg;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use super::Handler;

const TOTAL_SLOTS: usize = 20;

struct ChunkQueueItem {
    path: String,
    index: u64,
}

pub struct FileHandler {
    conf: Arc<AgentConf>,
    cert: Arc<ZCert>,
}

impl Handler<FileHandler> for FileHandler {
    fn new(conf: Arc<AgentConf>, cert: Arc<ZCert>) -> FileHandler {
        FileHandler {
            conf: conf,
            cert: cert,
        }
    }

    fn run(&self) -> Result<()> {
        println!("0");
        let upload_sock = try!(ZSock::new_sub(&format!("tcp://*:{}", self.conf.upload_port), None));
        println!("1");
        upload_sock.set_zap_domain("intecture");
        upload_sock.set_curve_server(true);
        upload_sock.set_rcvhwm(TOTAL_SLOTS as i32);
        self.cert.apply(&upload_sock);
println!("2");
        let download_sock = try!(ZSock::new_pub(&format!("tcp://*:{}", self.conf.download_port)));
println!("3");
        download_sock.set_zap_domain("intecture");
        download_sock.set_curve_server(true);
        self.cert.apply(&download_sock);

        let api_sock = try!(ZSock::new_pair("inproc://api_file_link"));
println!("4");
        let queue_sock = try!(ZSock::new_pull("inproc://slice_queue"));
        let queue_api_sock = try!(ZSock::new_push("inproc://slice_queue"));
        let queue_file_sock = try!(ZSock::new_push("inproc://slice_queue"));
println!("5");
        let files = Arc::new(RwLock::new(HashMap::new()));
        let files_c = files.clone();

        let mut available_slots = TOTAL_SLOTS;
        let chunk_queue: Arc<Mutex<Vec<ChunkQueueItem>>> = Arc::new(Mutex::new(Vec::new()));

        thread::spawn(move || {
            loop {
                if let Err(e) = handle_api_msg(&api_sock, &queue_api_sock, &files_c) {
                    // XXX This error should be logged.
                    let _ = Msg::send_err(&api_sock, e);
                }
            }
        });

        thread::spawn(move || {
            loop {
                // XXX This error should be logged.
                let _ = handle_download_msg(&download_sock, &queue_sock, &chunk_queue, &mut available_slots);
            }
        });

        loop {
            // XXX This error should be logged.
            let _ = handle_upload_msg(&upload_sock, &queue_file_sock, &files);
        }
    }
}

fn handle_api_msg(sock: &ZSock, queue_sock: &ZSock, files: &Arc<RwLock<HashMap<String, File>>>) -> Result<()> {
    let request = try!(Msg::expect_recv(&sock, 4, None, true));
    let path = try!(request.popstr()).unwrap();
    let hash = try!(request.popstr()).unwrap().parse::<u64>().unwrap();
    let size = try!(request.popstr()).unwrap().parse::<u64>().unwrap();
    let total_chunks = try!(request.popstr()).unwrap().parse::<u64>().unwrap();

    let mut opts = vec![];
    while let Ok(result) = request.popstr() {
        opts.push(result.unwrap());
    }

    let file = try!(File::new(&path, hash, size, total_chunks, convert_opt_args(opts)));
    for x in 0..total_chunks {
        try!(Msg::send(&queue_sock, vec!["QUEUE", &path, &x.to_string()]));
    }

    files.write().unwrap().insert(path, file);

    Ok(())
}

fn handle_download_msg(download_sock: &ZSock, queue_sock: &ZSock, chunk_queue: &Arc<Mutex<Vec<ChunkQueueItem>>>, available_slots: &mut usize) -> Result<()> {
    let request = try!(Msg::expect_recv(&queue_sock, 2, Some(2), true));
    let cmd = try!(request.popstr()).unwrap();
    let path = try!(request.popstr()).unwrap();

    match cmd.as_ref() {
        "READY" | "QUEUE" => {
            let mut queue = chunk_queue.lock().unwrap();

            if cmd == "QUEUE" {
                let request = try!(Msg::expect_recv(&queue_sock, 1, Some(1), false));

                queue.push(ChunkQueueItem {
                    path: path,
                    index: try!(request.popstr()).unwrap().parse::<u64>().unwrap(),
                });
            } else {
                *available_slots += 1
            }

            if *available_slots > 0 && queue.len() > 0 {
                let item = queue.remove(0);
                try!(Msg::send(download_sock, vec![
                    &item.path,
                    "Chk",
                    &item.index.to_string(),
                ]));
                *available_slots -= 1;
            }
        },
        "DONE" => {
            try!(Msg::send(download_sock, vec![
                &path,
                "Done",
            ]));
        },
        "ERR" => {
            let request = try!(Msg::expect_recv(&queue_sock, 1, Some(1), false));

            try!(Msg::send(download_sock, vec![
                &path,
                "Err",
                &try!(request.popstr()).unwrap(),
            ]));
        },
        _ => unimplemented!(),
    }

    Ok(())
}

fn handle_upload_msg(upload_sock: &ZSock, queue_sock: &ZSock, files: &Arc<RwLock<HashMap<String, File>>>) -> Result<()> {
    let request = try!(Msg::expect_recv(&upload_sock, 3, Some(3), true));
    let path = try!(request.popstr()).unwrap();
    let chunk_index = try!(request.popstr()).unwrap().parse::<u64>().unwrap();
    let chunk = try!(request.popbytes());

    if files.read().unwrap().contains_key(&path) {
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
                            try!(Msg::send(&queue_sock, vec!["DONE", &path]));
                        },
                        Err(e) => try!(Msg::send(&queue_sock, vec!["ERR", &path, e.description()])),
                    }
                }
            },
            Err(e) => {
                if can_retry {
                    try!(Msg::send(&queue_sock, vec!["QUEUE", &path, &chunk_index.to_string()]));
                } else {
                    files.write().unwrap().remove(&path);
                    try!(Msg::send(&queue_sock, vec!["ERR", &path, e.description()]));
                }
            },
        }

        try!(Msg::send(&queue_sock, vec!["READY"]));
    }

    Ok(())
}
