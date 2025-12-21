use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use configuration::{model::target::IcmpTarget, DEFAULT_SOURCE};
use serde::Serialize;
use tokio::{net::lookup_host, process::Command};
use tracing::field;

use crate::ScrapeError;

/// Sanitize IP address to prevent command injection
/// Only allows valid IPv4 format: digits and dots
#[tracing::instrument(level = "debug", skip(ip), fields(ip_len = ip.len()))]
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
#[tracing::instrument(level = "debug", fields(timeout_sec = timeout))]
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
    #[tracing::instrument(level = "debug", skip(self))]
    pub fn to_logfmt(&self) -> String {
        format!("up={} duration={:?}", self.up, self.duration)
    }
}

#[tracing::instrument(
    level = "info",
    skip(target),
    fields(ipv4 = field::Empty, fqdn = field::Empty, timeout_sec = target.timeout_sec)
)]
pub async fn ping_target(
    target: &IcmpTarget,
) -> Result<(Ipv4Addr, Duration), Box<dyn std::error::Error>> {
    let span = tracing::Span::current();

    let ip = if let Some(ipv4) = &target.ipv4 {
        span.record("ipv4", tracing::field::display(ipv4));

        Ipv4Addr::from_str(ipv4)?
    } else if let Some(fqdn) = &target.fqdn {
        span.record("fqdn", tracing::field::display(fqdn));

        let ip = resolve_ip_from_url(fqdn).await?;
        span.record("ipv4", tracing::field::display(ip));

        ip
    } else {
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
            log::debug!("source={} event=ping_complete host={} status=success", DEFAULT_SOURCE, ip);
            Ok((ip, start.elapsed()))
        }
        Err(err) => {
            log::error!("source={} event=ping_complete status=failed err={}", DEFAULT_SOURCE, err.to_string());
            Err(Box::new(ScrapeError::NetworkError(format!("host {:?} not reachable", target))))
        }
    }
}

#[tracing::instrument(level = "debug", fields(fqdn = %fqdn))]
async fn resolve_ip_from_url(fqdn: &str) -> Result<Ipv4Addr, ScrapeError> {
    let lookup_result = lookup_host((fqdn, 0)).await.map_err(|err| {
        log::error!("source={} event=dns_lookup_failed fqdn={} err={}", DEFAULT_SOURCE, fqdn, err);
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
