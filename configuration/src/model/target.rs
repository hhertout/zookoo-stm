use std::collections::HashMap;

use serde::Deserialize;

use crate::model::scrape_interval::ScrapeInterval;

fn default_scrape_interval() -> ScrapeInterval {
    return ScrapeInterval::M1;
}

fn default_follow_redirect() -> bool {
    return false;
}

fn default_method() -> String {
    return String::from("GET");
}

fn default_status_code() -> u16 {
    return 200;
}

fn default_skip_tls() -> bool {
    return false;
}

#[derive(Debug, Deserialize)]
pub struct HttpConfiguration {
    pub targets: Vec<HttpTarget>,
}

#[derive(Debug, Deserialize)]
pub struct HttpTarget {
    #[serde(default = "default_method")]
    pub method: String,
    pub url: String,
    #[serde(default = "default_status_code")]
    pub expected_status_code: u16,
    pub headers: Option<HashMap<String, String>>,
    pub labels: Option<HashMap<String, String>>,
    pub auth: Option<AuthConfiguration>,
    #[serde(default = "default_scrape_interval")]
    pub scrape_interval: ScrapeInterval,
    #[serde(default = "default_follow_redirect")]
    pub follow_redirect: bool,
    #[serde(default = "default_skip_tls")]
    pub skip_tls: bool,
}

#[derive(Debug, Deserialize)]
pub struct AuthConfiguration {
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IcmpConfiguration {
    pub targets: Vec<IcmpTarget>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IcmpTarget {
    pub ipv4: Option<String>,
    pub fqdn: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    #[serde(default = "default_scrape_interval")]
    pub scrape_interval: ScrapeInterval,
}
