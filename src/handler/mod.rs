// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

mod api;
mod file;

use config::agent::AgentConf;
use czmq::ZCert;
use error::Result;
use std::sync::Arc;
pub use self::api::ApiHandler;
pub use self::file::FileHandler;

pub trait Handler<T> {
    fn new(conf: Arc<AgentConf>, cert: Arc<ZCert>) -> T;
    fn run(&self) -> Result<()>;
}
