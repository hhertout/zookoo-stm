use std::collections::HashMap;

use serde::Deserialize;

use crate::model::scrape_interval::ScrapeInterval;

#[derive(Debug, Clone, Deserialize)]
pub struct Discovery {
    pub file: Option<HashMap<String, DiscoveryFile>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryFile {
    pub path: Vec<String>,
    pub labels: Option<HashMap<String, String>>,
    pub scrape_interval: Option<ScrapeInterval>,
    pub probe_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryFileTarget {
    pub path: Vec<String>,
    pub labels: Option<HashMap<String, String>>,
    pub scrape_interval: Option<ScrapeInterval>,
}
