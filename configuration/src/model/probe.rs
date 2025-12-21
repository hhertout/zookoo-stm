use serde::Deserialize;
use std::collections::HashMap;

use crate::model::target;

#[derive(Debug, Deserialize)]
pub struct ProbeConfiguration {
    pub http: Option<HashMap<String, target::HttpConfiguration>>,
    pub icmp: Option<HashMap<String, target::IcmpConfiguration>>,
    pub tcp: Option<HashMap<String, target::TcpConfiguration>>,
}
