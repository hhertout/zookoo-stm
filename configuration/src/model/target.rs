use std::collections::HashMap;

use serde::Deserialize;

use crate::model::scrape_interval::ScrapeInterval;

fn default_scrape_interval() -> ScrapeInterval {
    ScrapeInterval::S30
}

fn default_follow_redirect() -> bool {
    false
}

fn default_method() -> String {
    String::from("GET")
}

fn default_status_code() -> u16 {
    200
}

fn default_timeout() -> u16 {
    15
}

fn default_skip_tls() -> bool {
    false
}

#[derive(Debug, Deserialize)]
pub struct HttpConfiguration {
    pub targets: Option<Vec<HttpTarget>>,
    pub target_from: Option<String>,
    pub forward_to: Vec<String>,
    #[serde(default = "default_scrape_interval")]
    pub scrape_interval: ScrapeInterval,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpTarget {
    #[serde(default = "default_method")]
    pub method: String,
    pub url: String,
    #[serde(default = "default_status_code")]
    pub expected_status_code: u16,
    pub headers: Option<HashMap<String, String>>,
    pub labels: Option<HashMap<String, String>>,
    pub auth: Option<AuthConfiguration>,
    #[serde(default = "default_timeout")]
    pub timeout_sec: u16,
    #[serde(default = "default_follow_redirect")]
    pub follow_redirect: bool,
    #[serde(default = "default_skip_tls")]
    pub skip_tls: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AuthConfiguration {
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IcmpConfiguration {
    pub targets: Option<Vec<IcmpTarget>>,
    pub target_from: Option<String>,
    pub forward_to: Vec<String>,
    #[serde(default = "default_scrape_interval")]
    pub scrape_interval: ScrapeInterval,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IcmpTarget {
    pub ipv4: Option<String>,
    pub fqdn: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    #[serde(default = "default_timeout")]
    pub timeout_sec: u16,
}
