use std::collections::HashMap;

use serde::Deserialize;

use crate::model::{RefreshInterval, scrape_interval::ScrapeInterval};

pub fn default_refresh_interval() -> RefreshInterval {
    RefreshInterval::H1
}

#[derive(Debug, Clone, Deserialize)]
pub struct Discovery {
    pub file: Option<HashMap<String, DiscoveryFile>>,
    pub api: Option<HashMap<String, DiscoveryApi>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryFile {
    pub path: String,
    pub labels: Option<HashMap<String, String>>,
    pub scrape_interval: Option<ScrapeInterval>,
    pub probe_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryApi {
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub basic_auth: Option<String>,
    pub bearer: Option<String>,
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: RefreshInterval,
}
