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

pub struct File {
    path: String,
    tmp_path: String,
    tmp_file: fs::File,
    hash: SipHasher,
    origin_hash: u64,
    size: u64,
    last_chunk: u64,
    cached_chunks: HashMap<u64, Vec<u8>>,
    failed_chunks: u8,
}

impl File {
    fn tmp_filename(path: &str) -> String {
        let mut suffix: u16 = 0;

        loop {
            let mut path_buf = PathBuf::new();
            path_buf.push(path);
            path_buf.push("_tmp");
            path_buf.push(&suffix.to_string());

            if !path_buf.exists() {
                return path_buf.into_os_string().into_string().unwrap();
            }

            suffix += 1;
        }
    }

    pub fn new(path: &str, hash: u64, size: u64) -> Result<File> {
        let path_buf = PathBuf::from(path);

        if path_buf.is_dir() {
            return Err(Error::FileIsDirectory);
        }

        let tmp_path = Self::tmp_filename(path);
        let tmp_file = try!(fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&tmp_path));

        Ok(File {
            path: path.to_string(),
            tmp_path: tmp_path,
            tmp_file: tmp_file,
            hash: SipHasher::new(),
            origin_hash: hash,
            size: size,
            last_chunk: 0,
            cached_chunks: HashMap::new(),
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
        // If length is zero, this is the last chunk in the file
        if chunk.len() == 0 {
            try!(self.install());
        } else if index == self.last_chunk + 1 {
            try!(self.tmp_file.write_all(&chunk));
            self.hash.write(&chunk);
            self.last_chunk = index;

            // Write any cached chunks that are next in line
            let mut next_chunk = index + 1;
            while self.cached_chunks.contains_key(&next_chunk) {
                try!(self.tmp_file.write_all(&self.cached_chunks.remove(&next_chunk).unwrap()));
                self.hash.write(&chunk);
                self.last_chunk = next_chunk;
                next_chunk += 1;
            }
        } else {
            self.cached_chunks.insert(index, chunk);
        }

        Ok(())
    }

    fn install(&mut self) -> Result<()> {
        let meta = try!(self.tmp_file.metadata());

        if meta.len() != self.size {
            return Err(Error::FileSizeMismatch);
        }

        if self.hash.finish() != self.origin_hash {
            return Err(Error::FileHashMismatch);
        }

        try!(fs::rename(&self.tmp_path, &self.path));

        Ok(())
    }

    pub fn can_retry(&self) -> bool {
        self.failed_chunks < MAX_ATTEMPTS
    }
}
