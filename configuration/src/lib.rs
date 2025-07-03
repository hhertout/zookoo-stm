use std::io::ErrorKind::InvalidInput;
use std::{fs, io::Error};

use crate::model::Configuration;

pub mod model;

pub trait Parse<T> {
    fn parse_from_file(&self, file_path: &str) -> Result<T, Error>;
}

pub struct ConfigParser {}

impl ConfigParser {
    pub fn new() -> Self {
        ConfigParser {}
    }
}

impl Parse<Configuration> for ConfigParser {
    fn parse_from_file<'a>(&self, file_path: &'a str) -> Result<Configuration, Error> {
        let content = fs::read_to_string(file_path)?;
        let config: Configuration = match toml::from_str(&content) {
            Ok(conf) => conf,
            Err(err) => return Err(Error::new(InvalidInput, format!("{}", err.to_string()))),
        };

        Ok(config)
    }
}
