// Copyright 2015 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use std::{convert, error, fmt, io, result};
#[cfg(feature = "remote-run")]
use zmq;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    FileSizeMismatch,
    FileHashMismatch,
    FileIsDirectory,
    InvalidArgsCount,
    Io(io::Error),
    MissingConf,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::FileSizeMismatch => write!(f, "File size does not match expected size"),
            Error::FileHashMismatch => write!(f, "File hash does not match expected size"),
            Error::FileIsDirectory => write!(f, "Expected file but found directory"),
            Error::InvalidArgsCount => write!(f, "Invalid number of args provided"),
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::MissingConf => write!(f, "Cannot open Agent config"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::FileSizeMismatch => "File size does not match expected size",
            Error::FileHashMismatch => "File hash does not match expected size",
            Error::FileIsDirectory => "Expected file but found directory",
            Error::InvalidArgsCount => "Invalid number of args provided",
            Error::Io(ref e) => e.description(),
            Error::MissingConf => "Cannot open Agent config",
        }
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

#[cfg(feature = "remote-run")]
#[derive(Debug)]
pub struct MissingFrame {
    name: String,
    order: u8,
}

#[cfg(feature = "remote-run")]
impl MissingFrame {
    pub fn new(name: &str, order: u8) -> MissingFrame {
        MissingFrame {
            name: name.to_string(),
            order: order,
        }
    }
}
