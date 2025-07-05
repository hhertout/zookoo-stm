use std::io::Error;

use opentelemetry::Context;

use crate::config::target::HttpTarget;

pub mod http;

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    HTTP,
    HTTPS,
}

impl ToString for TargetType {
    fn to_string(&self) -> String {
        match self {
            TargetType::HTTP => String::from("HTTP"),
            TargetType::HTTPS => String::from("HTTPS"),
        }
    }
}

pub trait Scraping {
    fn scrape(&self) -> impl Future<Output = Result<(), Error>> + Send;
    fn send_request(
        &self,
        target: &HttpTarget,
        cx: Context,
    ) -> impl Future<Output = Result<(), Error>> + Send;
}
