// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use czmq;
use inapi;
use rustc_serialize::json;
use std::{convert, error, fmt, io, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Czmq(czmq::Error),
    // XXX This should disappear once ZMQ messages are properly typed
    FileError(String),
    FileHashMismatch,
    FileIsDirectory,
    FileSizeMismatch,
    Inapi(inapi::Error),
    InvalidArgsCount,
    InvalidEndpoint,
    InvalidStatus,
    Io(io::Error),
    JsonEncoder(json::EncoderError),
    MissingConf,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Czmq(ref e) => write!(f, "CZMQ error: {}", e),
            Error::FileError(ref e) => write!(f, "{}", e),
            Error::FileHashMismatch => write!(f, "File hash does not match expected hash"),
            Error::FileIsDirectory => write!(f, "Expected file but found directory"),
            Error::FileSizeMismatch => write!(f, "File size does not match expected size"),
            Error::Inapi(ref e) => write!(f, "Intecture API error: {}", e),
            Error::InvalidArgsCount => write!(f, "Invalid number of args provided"),
            Error::InvalidEndpoint => write!(f, "Invalid endpoint"),
            Error::InvalidStatus => write!(f, "Invalid reply status"),
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::JsonEncoder(ref e) => write!(f, "JSON encoder error: {}", e),
            Error::MissingConf => write!(f, "Cannot open Agent config"),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Czmq(ref e) => e.description(),
            Error::FileError(ref e) => e,
            Error::FileHashMismatch => "File hash does not match expected hash",
            Error::FileIsDirectory => "Expected file but found directory",
            Error::FileSizeMismatch => "File size does not match expected size",
            Error::Inapi(ref e) => e.description(),
            Error::InvalidArgsCount => "Invalid number of args provided",
            Error::InvalidEndpoint => "Invalid endpoint",
            Error::InvalidStatus => "Invalid reply status",
            Error::Io(ref e) => e.description(),
            Error::JsonEncoder(ref e) => e.description(),
            Error::MissingConf => "Cannot open Agent config",
        }
    }
}

impl convert::From<czmq::Error> for Error {
    fn from(err: czmq::Error) -> Error {
        Error::Czmq(err)
    }
}

impl convert::From<inapi::Error> for Error {
    fn from(err: inapi::Error) -> Error {
        Error::Inapi(err)
    }
}

impl convert::From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl convert::From<json::EncoderError> for Error {
    fn from(err: json::EncoderError) -> Error {
        Error::JsonEncoder(err)
    }
}
