use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use configuration::model::target::IcmpTarget;
use opentelemetry::{
    Context, KeyValue,
    trace::{Status, TraceContextExt},
};
use serde::Serialize;
use tokio::{net::lookup_host, process::Command};

use crate::{ScrapeError, observability::child_span_from_context};

/// Sanitize IP address to prevent command injection
/// Only allows valid IPv4 format: digits and dots
pub(super) fn sanitize_ip(ip: &str) -> Result<String, ScrapeError> {
    if ip.is_empty() {
        return Err(ScrapeError::InvalidInput("IP address cannot be empty".to_string()));
    }

    if ip.chars().all(|c| c.is_ascii_digit() || c == '.') {
        Ok(ip.to_string())
    } else {
        Err(ScrapeError::InvalidInput(format!("Invalid IP address format: {}", ip)))
    }
}

/// Sanitize timeout value to prevent command injection
/// Only allows positive integers between 1 and 3600 (1 hour max)
pub(super) fn sanitize_timeout(timeout: u16) -> Result<String, ScrapeError> {
    if timeout > 0 && timeout <= 3600 {
        Ok(timeout.to_string())
    } else {
        Err(ScrapeError::InvalidInput(format!(
            "Timeout must be between 1 and 3600 seconds, got: {}",
            timeout
        )))
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct IcmpMetrics {
    pub target: String,
    pub up: u8,
    pub duration: Duration,
}

impl IcmpMetrics {
    pub fn to_logfmt(&self) -> String {
        format!("up={} duration={:?}", self.up, self.duration)
    }
}

pub async fn ping_target(
    target: &IcmpTarget,
    ctx: Context,
) -> Result<(Ipv4Addr, Duration), Box<dyn std::error::Error>> {
    let cx_with_span = child_span_from_context("ping_target", ctx.clone(), vec![]);
    let span_ref = cx_with_span.span();

    let ip = if let Some(ipv4) = &target.ipv4 {
        span_ref.set_attribute(KeyValue::new("ipv4", ipv4.clone()));

        let ip = Ipv4Addr::from_str(ipv4)?;

        span_ref.set_status(Status::Ok);
        ip
    } else if let Some(fqdn) = &target.fqdn {
        span_ref.set_attribute(KeyValue::new("fqdn", fqdn.clone()));

        let ip = resolve_ip_from_url(fqdn).await?;
        span_ref.set_attribute(KeyValue::new("ipv4", ip.to_string()));

        ip
    } else {
        span_ref.set_status(Status::Error { description: "ipv4 & fqdn are unset".into() });
        return Err(Box::new(ScrapeError::TypeError("ipv4 & fqdn are unset".to_string())));
    };

    // Sanitize inputs before passing to command
    let sanitized_ip = sanitize_ip(&ip.to_string())?;
    let sanitized_timeout = sanitize_timeout(target.timeout_sec)?;

    let start = Instant::now();
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "1", "-t", &sanitized_timeout, &sanitized_ip])
        .output()
        .await;

    match output {
        Ok(_) => {
            log::debug!("event=ping_complete host={:?} status=success", ip);
            span_ref.set_status(Status::Ok);
            Ok((ip, start.elapsed()))
        }
        Err(err) => {
            log::error!("event=ping_complete status=failed err={:?}", err.to_string());
            span_ref.set_status(Status::Error {
                description: format!("host {:?} not reachable", target).into(),
            });
            Err(Box::new(ScrapeError::NetworkError(format!("host {:?} not reachable", target))))
        }
    }
}

async fn resolve_ip_from_url(fqdn: &str) -> Result<Ipv4Addr, ScrapeError> {
    let lookup_result = lookup_host((fqdn, 0)).await.map_err(|err| {
        log::error!("{}", err);
        ScrapeError::LookupFailed
    })?;

    let ipv4_addr = lookup_result
        .filter_map(|addr| match addr {
            SocketAddr::V4(v4_addr) => Some(*v4_addr.ip()),
            SocketAddr::V6(_) => None,
        })
        .next()
        .ok_or(ScrapeError::InvalidInput(format!("no ipv4 found for target = {:?}", fqdn)))?;
    Ok(ipv4_addr)
}
