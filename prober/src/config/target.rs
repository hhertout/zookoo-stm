use std::collections::HashMap;

use serde::Deserialize;

use crate::config::scrape_interval::ScrapeInterval;

fn default_scrape_interval() -> ScrapeInterval {
    return ScrapeInterval::M5;
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

#[derive(Debug, Clone)]
pub struct HttpConfiguration {
    pub target_file: Option<Vec<String>>,
    pub targets: Option<Vec<HttpTarget>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpTarget {
    #[serde(default = "default_method")]
    pub method: String,
    #[serde(default = "default_status_code")]
    pub expected_status_code: u16,
    pub url: String,
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

#[derive(Debug, Clone, Deserialize)]
pub struct AuthConfiguration {
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IcmpConfiguration {
    pub target_file: Option<Vec<String>>,
    pub targets: Option<Vec<IcmpTarget>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IcmpTarget {
    pub ipv4: Option<String>,
    pub labels: Option<HashMap<String, String>>,
    #[serde(default = "default_scrape_interval")]
    pub scrape_interval: ScrapeInterval,
}

impl From<configuration::model::target::IcmpConfiguration> for IcmpConfiguration {
    fn from(value: configuration::model::target::IcmpConfiguration) -> Self {
        IcmpConfiguration {
            target_file: value.target_file,
            targets: value
                .targets
                .map(|targets| targets.into_iter().map(IcmpTarget::from).collect()),
        }
    }
}

impl From<configuration::model::target::IcmpTarget> for IcmpTarget {
    fn from(value: configuration::model::target::IcmpTarget) -> Self {
        IcmpTarget {
            ipv4: value.ipv4,
            labels: value.labels,
            scrape_interval: ScrapeInterval::from(value.scrape_interval),
        }
    }
}

impl From<configuration::model::target::HttpConfiguration> for HttpConfiguration {
    fn from(value: configuration::model::target::HttpConfiguration) -> Self {
        HttpConfiguration {
            target_file: value.target_file,
            targets: value
                .targets
                .map(|targets| targets.into_iter().map(HttpTarget::from).collect()),
        }
    }
}

impl From<configuration::model::target::HttpTarget> for HttpTarget {
    fn from(value: configuration::model::target::HttpTarget) -> Self {
        HttpTarget {
            expected_status_code: value.expected_status_code,
            method: value.method,
            url: value.url,
            headers: value.headers,
            labels: value.labels,
            auth: value.auth.map(AuthConfiguration::from),
            scrape_interval: ScrapeInterval::from(value.scrape_interval),
            follow_redirect: value.follow_redirect,
            skip_tls: value.skip_tls,
        }
    }
}

impl From<configuration::model::target::AuthConfiguration> for AuthConfiguration {
    fn from(value: configuration::model::target::AuthConfiguration) -> Self {
        AuthConfiguration {
            username: value.username,
            password: value.password,
            bearer: value.bearer,
        }
    }
}
