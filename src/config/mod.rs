// Copyright 2015-2016 Intecture Developers. See the COPYRIGHT file at the
// top-level directory of this distribution and at
// https://intecture.io/COPYRIGHT.
//
// Licensed under the Mozilla Public License 2.0 <LICENSE or
// https://www.tldrlegal.com/l/mpl-2.0>. This file may not be copied,
// modified, or distributed except according to those terms.

pub mod agent;

use rustc_serialize::{Decodable, Encodable, json};
use std::{io, convert, fs};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub trait Config {
    type ConfigFile: Decodable + Encodable;

	fn load(file_path: &Path) -> Result<Self::ConfigFile, ConfigError> {
		let mut config_file = try!(fs::File::open(&file_path));
		let mut config_content = String::new();
		try!(config_file.read_to_string(&mut config_content));

		let config: Self::ConfigFile = try!(json::decode(&config_content));
		Ok(config)
	}

	fn save(config: &Self::ConfigFile, file_path: &Path) -> Result<(), ConfigError> {
        let mut file = try!(File::create(file_path));
        let json = try!(json::encode(config));

        try!(file.write_all(&json.as_bytes()));
        Ok(())
	}
}

#[derive(Debug)]
pub enum ConfigError {
	IoError(io::Error),
	JsonDecoderError(json::DecoderError),
	JsonEncoderError(json::EncoderError),
}

impl convert::From<io::Error> for ConfigError {
	fn from(err: io::Error) -> ConfigError {
		ConfigError::IoError(err)
	}
}

impl convert::From<json::DecoderError> for ConfigError {
	fn from(err: json::DecoderError) -> ConfigError {
		ConfigError::JsonDecoderError(err)
	}
}

impl convert::From<json::EncoderError> for ConfigError {
	fn from(err: json::EncoderError) -> ConfigError {
		ConfigError::JsonEncoderError(err)
	}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs::{File, remove_file};
    use std::io::{Read, Write};

    #[derive(RustcDecodable, RustcEncodable)]
    struct TestConf {
        pub attribute: i32,
    }

    impl Config for TestConf {
        type ConfigFile = TestConf;
    }

    #[test]
    fn test_conf_load() {
        let test_path = Path::new("test_conf_load.json");

        let mut file = File::create(&test_path).unwrap();
        file.write_all("{\"attribute\":123}".as_bytes()).unwrap();

        let test_conf = TestConf::load(&test_path).unwrap();
        assert_eq!(test_conf.attribute, 123);

        remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_conf_save() {
        let test_conf = TestConf {
            attribute: 123
        };

        let test_path = Path::new("test_conf_save.json");

        TestConf::save(&test_conf, &test_path).unwrap();

        let mut file = File::open(&test_path).unwrap();
		let mut content = String::new();
		file.read_to_string(&mut content).unwrap();

		assert_eq!(content, "{\"attribute\":123}");

        remove_file(&test_path).unwrap();
    }
}