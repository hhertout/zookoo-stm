use std::{
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use opentelemetry::{
    Context, KeyValue,
    trace::{Status, TraceContextExt},
};
use serde::Serialize;
use tokio::{net::lookup_host, process::Command};
use url::Url;

use crate::{child_span_from_context, config::target::IcmpTarget, target::ScrapeError};

#[derive(Debug, Clone, Serialize)]
pub struct IcmpMetrics {
    pub up: u8,
    pub duration: Duration,
}

pub async fn ping_target(
    target: &IcmpTarget,
    ctx: Context,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let span_attr = vec![KeyValue::new(
        "ipv4",
        target.ipv4.clone().unwrap_or("unset".to_string()).clone(),
    )];
    let cx_with_span = child_span_from_context("ping_target", ctx.clone(), span_attr);
    let span_ref = cx_with_span.span();

    // check if ip is valid
    let ip = if let Some(ip4) = &target.ipv4 {
        let ip = Ipv4Addr::from_str(&ip4)?;
        let span_ref = cx_with_span.span();
        span_ref.set_attribute(KeyValue::new("ipv4", ip4.to_string()));

        ip
    } else {
        span_ref.set_status(Status::Error {
            description: "No IPV4 or Address set in the target".into(),
        });
        return Err(Box::new(ScrapeError::InvalidInput(
            "No IPV4 or Address set in the target".to_string(),
        )));
    };

    let start = Instant::now();
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "1", &ip.to_string()])
        .output()
        .await;

    match output {
        Ok(_) => Ok(start.elapsed()),
        Err(err) => {
            log::error!("ping failed {:?}", err.to_string());
            Err(Box::new(ScrapeError::NetworkError(format!(
                "host {:?} not reachable",
                target
            ))))
        }
    }
}

async fn resolve_ip_from_url(target: &IcmpTarget) -> Result<Ipv4Addr, ScrapeError> {
    // TODO: ADAPT THIS TO THIS ICMP DEFAULT PORT
    // NOT FUNCTIONAL
    // TO ADAPT !!
    let parsed_url = Url::parse(&target.ipv4.as_ref().unwrap())
        .map_err(|_| ScrapeError::InvalidUrl(target.ipv4.as_ref().unwrap().to_string()))?;

    let host = parsed_url.host_str().ok_or(ScrapeError::InvalidHost)?;

    let port = parsed_url
        .port()
        .unwrap_or_else(|| match parsed_url.scheme() {
            "https" => 443,
            "http" => 80,
            _ => 443,
        });

    let start = Instant::now();
    let lookup_result = lookup_host((host, port))
        .await
        .map_err(|_| ScrapeError::LookupFailed)?;

    let ipv4_addr = lookup_result
        .filter_map(|addr| match addr {
            SocketAddr::V4(v4_addr) => Some(*v4_addr.ip()),
            SocketAddr::V6(_) => None,
        })
        .next()
        .ok_or(ScrapeError::InvalidInput(format!(
            "no ipv4 found for target = {:?}",
            target
        )))?;

    let duration = start.elapsed();
    log::debug!("DNS resolution for {} took {:?}", host, duration);

    Ok(ipv4_addr)
}
