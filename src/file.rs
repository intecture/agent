// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use error::{Error, Result};
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, SipHasher, Hasher};
use std::io::Write;
use std::path::PathBuf;

const MAX_ATTEMPTS: u8 = 10;

enum ReplaceStrategy {
    Backup(String),
    Unlink,
}

pub struct File {
    path: String,
    exists: bool,
    replace_strategy: ReplaceStrategy,
    upload_path: String,
    upload_file: fs::File,
    hash: SipHasher,
    origin_hash: u64,
    size: u64,
    total_chunks: u64,
    written_chunks: Vec<u64>,
    queued_chunks: HashMap<u64, Vec<u8>>,
    failed_chunks: u8,
}

impl File {
    fn get_unique_filename(path: &str, suffix: &str) -> String {
        let mut counter: u16 = 0;

        let mut base_path = path.to_string();
        base_path.push_str(suffix);

        let mut counter_path = base_path.clone();

        loop {
            if !PathBuf::from(&counter_path).exists() {
                return counter_path;
            }

            counter_path = format!("{}{}", base_path, counter);
            counter += 1;
        }
    }

    pub fn new(path: &str, hash: u64, size: u64, total_chunks: u64) -> Result<File> {
        let path_buf = PathBuf::from(path);

        if path_buf.is_dir() {
            return Err(Error::FileIsDirectory);
        }

        // XXX Check disk space?

        let tmp_path = Self::get_unique_filename(path, "_upload");
        let tmp_file = try!(fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&tmp_path));

        // XXX Placeholder for value from options vector.
        let replace_strategy = ReplaceStrategy::Backup("_moo".to_string());

        Ok(File {
            path: path.to_string(),
            exists: path_buf.exists(),
            replace_strategy: replace_strategy,
            upload_path: tmp_path,
            upload_file: tmp_file,
            hash: SipHasher::new(),
            origin_hash: hash,
            size: size,
            total_chunks: total_chunks,
            written_chunks: Vec::new(),
            queued_chunks: HashMap::new(),
            failed_chunks: 0,
        })
    }

    pub fn write(&mut self, index: u64, chunk: Vec<u8>) -> Result<()> {
        let result = self.do_write(index, chunk);

        if result.is_err() {
            self.failed_chunks += 1;
        }

        result
    }

    fn do_write(&mut self, index: u64, chunk: Vec<u8>) -> Result<()> {
        if index == self.written_chunks.len() as u64 {
            try!(self.upload_file.write_all(&chunk));
            self.hash.write(&chunk);
            self.written_chunks.push(index);

            // Write any cached chunks that are next in line
            let mut next_chunk = index + 1;
            while self.queued_chunks.contains_key(&next_chunk) {
                try!(self.upload_file.write_all(&self.queued_chunks.remove(&next_chunk).unwrap()));
                self.hash.write(&chunk);
                self.written_chunks.push(next_chunk);
                next_chunk += 1;
            }
        } else {
            self.queued_chunks.insert(index, chunk);
        }

        Ok(())
    }

    pub fn install(&mut self) -> Result<()> {
        let meta = try!(self.upload_file.metadata());

        if meta.len() != self.size {
            return Err(Error::FileSizeMismatch);
        }

        if self.hash.finish() != self.origin_hash {
            return Err(Error::FileHashMismatch);
        }

        // Backup/unlink existing file if exists
        if self.exists {
            let bk_path: String;

            match &self.replace_strategy {
                &ReplaceStrategy::Backup(ref suffix) => {
                    bk_path = Self::get_unique_filename(&self.path, &suffix);
                    try!(fs::rename(&self.path, &bk_path));
                },
                &ReplaceStrategy::Unlink => {
                    bk_path = Self::get_unique_filename(&self.path, "_bk");
                    try!(fs::rename(&self.path, &bk_path));
                },
            }

            match fs::rename(&self.upload_path, &self.path) {
                // XXX This is an inelegant solution. ReplaceStrategy
                // logic should be grouped together.
                Ok(_) => match self.replace_strategy {
                    ReplaceStrategy::Unlink => try!(fs::remove_file(&bk_path)),
                    _ => (),
                },
                Err(e) => {
                    try!(fs::rename(&bk_path, &self.path));
                    try!(fs::remove_file(&self.upload_path));
                    return Err(Error::from(e));
                }
            }
        } else {
            try!(fs::rename(&self.upload_path, &self.path));
        }

        Ok(())
    }

    pub fn is_finished(&self) -> bool {
        self.written_chunks.len() == self.total_chunks as usize
    }

    pub fn can_retry(&self) -> bool {
        self.failed_chunks < MAX_ATTEMPTS
    }
}
