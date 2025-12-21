//! # Configuration crate
//!
//! This crate is responsible of the parsing of the configuration file
//!
use std::error::Error;
use std::fs;

use crate::model::Configuration;

pub mod model;

pub struct HCL;

pub const DEFAULT_SOURCE: &str = "zookoo";

/// Trait to define parser behavior
pub trait ParserType {
    fn parse(content: &str) -> Result<Configuration, Box<dyn Error>>;
}

impl ParserType for HCL {
    fn parse(content: &str) -> Result<Configuration, Box<dyn Error>> {
        hcl::from_str(content).map_err(|e| e.into())
    }
}

/// Parse trait for configuration files.
/// This trait defines a method to parse a configuration file from a given file path.
/// It is generic over the type `T`, allowing for flexibility in the types of configurations that can be parsed.
/// The `parse_from_file` method reads the content of the file and deserializes it into the specified type `T`.
/// It returns a result containing the deserialized value or an error if the operation fails.
/// This trait can be implemented for various configuration types, enabling easy parsing
/// of configuration files in different formats, such as JSON or TOML.
pub trait Parse<T> {
    /// Parse a configuration file from the given file path with the specified parser type.
    fn parse_from_file<P: ParserType>(&self, file_path: &str) -> Result<T, Box<dyn Error>>;
}

pub struct ConfigParser;
impl Parse<Configuration> for ConfigParser {
    /// Parse a configuration file from the given file path.
    /// This method reads the content of the file at the specified path and deserializes it
    /// into a `Configuration` object. It uses the `serde` library to perform the deserialization.
    /// If the file is successfully read and parsed, it returns a `Configuration` object.
    /// If there is an error during reading or parsing, it returns an error wrapped in a `Box<dyn Error>`.
    /// This allows for flexible error handling and makes it easy to integrate with other parts of the application.
    ///
    /// # Usage
    /// ```ignore
    /// let parser = ConfigParser;
    /// let config = parser.parse_from_file::<HCL>("config.hcl")?;
    /// ```
    fn parse_from_file<P: ParserType>(
        &self,
        file_path: &str,
    ) -> Result<Configuration, Box<dyn Error>> {
        let content = fs::read_to_string(file_path)?;
        P::parse(&content)
    }
}
