use std::collections::HashMap;

use serde::Deserialize;

use crate::model::scrape_interval::ScrapeInterval;

#[derive(Debug, Clone, Deserialize)]
pub struct Discovery {
    pub file: Option<DiscoveryFile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryFile {
    pub http: Option<Vec<DiscoveryFileTarget>>,
    pub icmp: Option<Vec<DiscoveryFileTarget>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiscoveryFileTarget {
    pub path: Vec<String>,
    pub labels: Option<HashMap<String, String>>,
    pub scrape_interval: Option<ScrapeInterval>,
}
