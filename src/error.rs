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
use zdaemon;
use zfilexfer;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Czmq(czmq::Error),
    Inapi(inapi::Error),
    Io(io::Error),
    JsonEncoder(json::EncoderError),
    ZDaemon(zdaemon::Error),
    ZFileXfer(zfilexfer::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Czmq(ref e) => write!(f, "CZMQ error: {}", e),
            Error::Inapi(ref e) => write!(f, "Intecture API error: {}", e),
            Error::Io(ref e) => write!(f, "IO error: {}", e),
            Error::JsonEncoder(ref e) => write!(f, "JSON encoder error: {}", e),
            Error::ZDaemon(ref e) => write!(f, "ZDaemon error: {}", e),
            Error::ZFileXfer(ref e) => write!(f, "ZFileXfer error: {}", e),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Czmq(ref e) => e.description(),
            Error::Inapi(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::JsonEncoder(ref e) => e.description(),
            Error::ZDaemon(ref e) => e.description(),
            Error::ZFileXfer(ref e) => e.description(),
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

impl convert::From<zdaemon::Error> for Error {
    fn from(err: zdaemon::Error) -> Error {
        Error::ZDaemon(err)
    }
}

impl convert::Into<zdaemon::Error> for Error {
    fn into(self) -> zdaemon::Error {
        zdaemon::Error::Generic(Box::new(self))
    }
}

impl convert::From<zfilexfer::Error> for Error {
    fn from(err: zfilexfer::Error) -> Error {
        Error::ZFileXfer(err)
    }
}
