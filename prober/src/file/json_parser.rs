use std::{fs, io::Error};

use crate::config::target::HttpTarget;

pub fn parse_json_from_file(path: String) -> Result<Vec<HttpTarget>, Error> {
    let content = fs::read_to_string(path)?;

    let parsed: Vec<HttpTarget> = serde_json::from_str(&content)
        .map_err(|e| Error::new(std::io::ErrorKind::InvalidData, e))?;

    Ok(parsed)
}
