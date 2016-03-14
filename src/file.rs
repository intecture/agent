// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
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
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAX_ATTEMPTS: u8 = 10;

pub enum FileOpts {
    BackupExistingFile(String),
}

pub fn convert_opt_args(args: Vec<String>) -> Vec<FileOpts> {
    let mut opts: Vec<FileOpts> = Vec::new();
    let mut args_i = args.into_iter();

    while args_i.len() > 0 {
        if let Some(arg) = args_i.next() {
            match arg.as_ref() {
                "OPT_BackupExistingFile" => {
                    if let Some(value) = args_i.next() {
                        opts.push(FileOpts::BackupExistingFile(value));
                    } else {
                        continue;
                    }
                },
                _ => continue,
            }
        }
    }

    opts
}

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
    queued_chunks: HashMap<u64, String>,
    queued_chunks_dir: Option<String>,
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

    pub fn new(path: &str, hash: u64, size: u64, total_chunks: u64, options: Vec<FileOpts>) -> Result<File> {
        let path_buf = PathBuf::from(path);

        if path_buf.is_dir() {
            return Err(Error::FileIsDirectory);
        }

        // XXX Check disk space?

        let upload_path = Self::get_unique_filename(path, "_upload");
        let upload_file = try!(fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&upload_path));

        let mut replace_strategy = ReplaceStrategy::Unlink;

        // Handle options
        for opt in options {
            match opt {
                FileOpts::BackupExistingFile(suffix) => replace_strategy = ReplaceStrategy::Backup(suffix),
            }
        }

        Ok(File {
            path: path.to_string(),
            exists: path_buf.exists(),
            replace_strategy: replace_strategy,
            upload_path: upload_path,
            upload_file: upload_file,
            hash: SipHasher::new(),
            origin_hash: hash,
            size: size,
            total_chunks: total_chunks,
            written_chunks: Vec::new(),
            queued_chunks: HashMap::new(),
            queued_chunks_dir: None,
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
            let mut next_index = index + 1;
            while self.queued_chunks.contains_key(&next_index) {
                let mut next_chunk: Vec<u8> = vec![];
                let chunk_path = self.queued_chunks.remove(&next_index).unwrap();
                let mut chunk_file = try!(fs::File::open(&chunk_path));
                try!(chunk_file.read_to_end(&mut next_chunk));
                try!(fs::remove_file(&chunk_path));

                try!(self.upload_file.write_all(&next_chunk));
                self.hash.write(&next_chunk);
                self.written_chunks.push(next_index);
                next_index += 1;
            }
        } else {
            if self.queued_chunks_dir.is_none() {
                let mut path = PathBuf::from(&self.path);
                let dir_name = path.file_name().unwrap().to_os_string().into_string().unwrap();
                path.set_file_name(format!(".{}", dir_name));
                self.queued_chunks_dir = Some(Self::get_unique_filename(path.to_str().unwrap(), "_chunks"));
                try!(fs::create_dir(self.queued_chunks_dir.as_ref().unwrap()));
            }

            let chunk_path = Self::get_unique_filename(&format!("{}/chunk{}", self.queued_chunks_dir.as_ref().unwrap(), index), "");
            let mut chunk_file = try!(fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&chunk_path));
            try!(chunk_file.write_all(&chunk));

            self.queued_chunks.insert(index, chunk_path);
        }

        Ok(())
    }

    pub fn install(&mut self) -> Result<()> {
        let result = self.do_install();

        // Cleanup
        if Path::new(&self.upload_path).exists() {
            try!(fs::remove_file(&self.upload_path));
        }

        if self.queued_chunks_dir.is_some() {
            try!(fs::remove_dir_all(self.queued_chunks_dir.as_ref().unwrap()));
        }

        result
    }

    fn do_install(&mut self) -> Result<()> {
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
                    bk_path = Self::get_unique_filename(&self.path, "_orig");
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

// XXX It is impossible to test this effectively without mocking FS
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_() {
//
//     }
// }
