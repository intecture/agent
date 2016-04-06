// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

use config::Config;
use error::{Error, Result};
use std::path::PathBuf;

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct AgentConf {
    pub server_cert: String,
    pub users_path: String,
    pub api_port: i32,
    pub upload_port: i32,
    pub download_port: i32,
}

impl AgentConf {
    pub fn load_path(paths: Option<&[&str]>) -> Result<AgentConf> {
        let default_paths = ["/usr/local/etc", "/etc"];
        let paths = paths.unwrap_or(&default_paths);

        for p in paths.iter() {
            let mut path = PathBuf::from(p);
            path.push("intecture");
            path.push("agent.json");

            match Self::load(&path) {
                Ok(conf) => return Ok(conf),
                Err(_) => continue,
            }
        }

        Err(Error::MissingConf)
    }
}

impl Config for AgentConf {
    type ConfigFile = AgentConf;
}

#[cfg(test)]
mod tests {
    use config::Config;
    use czmq::zsys_init;
    use std::fs::create_dir;
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn test_load_agent_conf() {
        zsys_init();

        let dir = TempDir::new("test_load_agent_conf").unwrap();

        assert!(AgentConf::load_path(Some(&["/fake/dir"])).is_err());

        let conf = AgentConf {
            server_cert: "/path/to/cert.crt".to_string(),
            users_path: "/path/to/users/certs".to_string(),
            api_port: 1,
            upload_port: 2,
            download_port: 3,
        };

        let mut dir_pathbuf = dir.path().to_path_buf();
        dir_pathbuf.push("intecture");
        create_dir(&dir_pathbuf).unwrap();

        dir_pathbuf.push("agent.json");
        AgentConf::save(&conf, dir_pathbuf.as_path()).unwrap();

        assert!(AgentConf::load_path(Some(&[dir.path().to_str().unwrap()])).is_ok());
    }
}
