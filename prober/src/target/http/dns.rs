use std::time::{Duration, Instant};

use serde::Serialize;
use tokio::net::lookup_host;

#[derive(Debug, Clone, Serialize)]
pub struct DnsMetrics {
    pub duration: Duration,
}

impl DnsMetrics {
    pub fn to_logfmt(&self) -> String {
        format!("dns_lookup_duration={}", self.duration.as_millis())
    }
}

pub async fn dns_lookup(url: &str) -> Result<DnsMetrics, Box<dyn std::error::Error>> {
    let parsed_url = url::Url::parse(url)?;
    let host = parsed_url.host_str().ok_or("Invalid host in URL")?;

    let port = 443;

    let start = Instant::now();

    let _ = lookup_host((host, port))
        .await?
        .find(|addr| addr.is_ipv4())
        .ok_or("no_ipv4")?;

    let total = start.elapsed();

    Ok(DnsMetrics { duration: total })
}
