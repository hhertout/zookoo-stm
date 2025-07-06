use opentelemetry::Context;

pub mod http;
pub mod icmp;

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    HTTP,
    HTTPS,
    IPV4,
}

impl ToString for TargetType {
    fn to_string(&self) -> String {
        match self {
            TargetType::HTTP => String::from("HTTP"),
            TargetType::HTTPS => String::from("HTTPS"),
            TargetType::IPV4 => String::from("ipv4"),
        }
    }
}

pub trait Scraping<T> {
    fn scrape(&self) -> impl Future<Output = Result<(), ()>> + Send;
    fn send_request(&self, target: &T, cx: Context) -> impl Future<Output = Result<(), ()>> + Send;
}
