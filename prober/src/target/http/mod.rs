use futures::future::join_all;
use std::io::{Error, ErrorKind};

use crate::{
    config::target::HttpTarget,
    metrics::{MetricExportable, Metrics, http_metrics::HttpRequestMetrics},
    target::{
        Scraping, TargetType,
        http::{
            dns::dns_lookup,
            request::http_request,
            tls::{TlsMetrics, inspect_tls},
        },
    },
};

pub(crate) mod dns;
pub(crate) mod request;
pub mod scrape;
pub(crate) mod tls;

#[derive(Clone)]
pub struct HttpScrapper {
    pub targets: Vec<HttpTarget>,
}

impl Scraping for HttpScrapper {
    async fn scrape(&self) -> Result<(), Error> {
        let futures = self.targets.iter().map(|target| self.send_request(target));

        let _ = join_all(futures).await;

        Ok(())
    }

    async fn send_request(&self, target: &HttpTarget) -> Result<(), Error> {
        let kind = match self.get_target_type(target.url.as_ref()) {
            Ok(target_type) => target_type,
            Err(err) => return Err(err),
        };

        log::info!(
            "event=request type={} target={}",
            kind.to_string(),
            target.url
        );

        if let Some(metrics) = self.build_http_metrics(kind, target).await {
            log::info!(
                "event=metrics target={} job=rustbox {} {} {}",
                target.url,
                metrics.dns.to_logfmt(),
                metrics.http.to_logfmt(),
                metrics
                    .tls
                    .as_ref()
                    .map(|t| t.to_logfmt())
                    .unwrap_or_default()
            );

            self.export_metrics(kind, target.url.clone(), Metrics::Http(metrics));
        }

        Ok(())
    }
}

impl HttpScrapper {
    async fn build_http_metrics(
        &self,
        kind: TargetType,
        target: &HttpTarget,
    ) -> Option<HttpRequestMetrics> {
        let dns_metrics = match dns_lookup(&target.url).await {
            Ok(m) => m,
            Err(err) => {
                log::error!("DNS lookup failed for url={} err={}", &target.url, err);
                return None;
            }
        };

        let tls_metrics = if kind == TargetType::HTTPS {
            match inspect_tls(&target.url).await {
                Ok(m) => Some(m),
                Err(err) => {
                    log::error!("TLS inspection failed for url={} err={}", &target.url, err);
                    Some(TlsMetrics::invalid())
                }
            }
        } else {
            None
        };

        let http_metrics = match http_request(target).await {
            Ok(m) => m,
            Err(err) => {
                log::error!("HTTP request failed for url={} err={}", &target.url, err);
                return None;
            }
        };

        Some(HttpRequestMetrics {
            dns: dns_metrics,
            http: http_metrics,
            tls: tls_metrics,
            labels: target.labels.clone(),
        })
    }

    fn export_metrics(&self, kind: TargetType, target: String, metrics: Metrics) {
        match (kind, metrics) {
            (TargetType::HTTP | TargetType::HTTPS, Metrics::Http(m)) => m.export(&target),
        };
    }

    fn get_target_type(&self, url: &str) -> Result<TargetType, Error> {
        if url.starts_with("https") {
            Ok(TargetType::HTTPS)
        } else if url.starts_with("http") {
            Ok(TargetType::HTTP)
        } else {
            log::error!("URL must start with http or https");
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "URL must start with http or https",
            ));
        }
    }
}
