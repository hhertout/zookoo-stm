use std::time::{Duration, Instant};

use opentelemetry::{
    Context, KeyValue,
    trace::{Status, TraceContextExt},
};
use serde::Serialize;
use tokio::net::lookup_host;

use crate::observability::child_span_from_context;

#[derive(Debug, Clone, Serialize)]
pub struct DnsMetrics {
    pub duration: Duration,
}

impl DnsMetrics {
    pub fn to_logfmt(&self) -> String {
        format!("dns_lookup_duration={}", self.duration.as_millis())
    }
}

pub async fn dns_lookup(url: &str, cx: Context) -> Result<DnsMetrics, Box<dyn std::error::Error>> {
    let span_attr = vec![KeyValue::new("url", url.to_string())];
    let cx_with_span = child_span_from_context("dns_lookup", cx.clone(), span_attr);
    let span_ref = cx_with_span.span();

    let parsed_url = url::Url::parse(url)?;
    let host = parsed_url.host_str().ok_or("Invalid host in URL")?;

    let port = 443;

    let start = Instant::now();

    let _ = lookup_host((host, port)).await?.find(|addr| addr.is_ipv4()).ok_or("no_ipv4")?;

    let total = start.elapsed();

    span_ref.set_status(Status::Ok);
    span_ref.end();

    Ok(DnsMetrics { duration: total })
}
