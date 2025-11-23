use futures::future::join_all;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{Context, KeyValue, global::ObjectSafeSpan, trace::Status};
use reqwest::Client;
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use crate::{
    config::target::HttpTarget,
    core::{MetricExportable, ScrapeError, Scraping},
    observability::{child_span_from_context, get_tracer, tracing_new_span},
    probes::http::{
        dns::dns_lookup,
        metrics::HttpRequestMetrics,
        request::http_request,
        tls::{TlsMetrics, inspect_tls},
    },
};

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    HTTP,
    HTTPS,
}

impl ToString for TargetType {
    fn to_string(&self) -> String {
        match self {
            TargetType::HTTP => String::from("HTTP"),
            TargetType::HTTPS => String::from("HTTPS"),
        }
    }
}

#[derive(Clone)]
pub struct HttpScraper {
    pub targets: Vec<HttpTarget>,
    client: Arc<Client>,
}

impl Scraping<HttpTarget> for HttpScraper {
    fn new(targets: Vec<HttpTarget>) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|e| {
                log::error!("Failed to build shared HTTP client: {}, using default", e);
                Client::new()
            });
        
        HttpScraper { 
            targets,
            client: Arc::new(client),
        }
    }

    async fn scrape(&self) -> Result<(), ScrapeError> {
        let mut span = tracing_new_span(get_tracer(), "scrape_target".to_string());
        span.set_attribute(KeyValue::new("type", "icmp".to_string()));
        let ctx = Context::current_with_span(span);
        let guard = ctx.clone().attach();

        let futures = self.targets.iter().map(|target| {
            let ctx = ctx.clone();
            self.send_request(target, ctx)
        });

        drop(guard);
        let _ = join_all(futures).await;

        Ok(())
    }

    async fn send_request(&self, target: &HttpTarget, ctx: Context) -> Result<(), ScrapeError> {
        let span_attr = vec![
            KeyValue::new("url", target.url.clone()),
            KeyValue::new("http_method", target.method.clone()),
            KeyValue::new(
                "expected_status_code",
                target.expected_status_code.clone().to_string(),
            ),
        ];
        let ctx_with_span = child_span_from_context("send_request", ctx.clone(), span_attr);

        let kind = match self.get_target_type(target.url.as_ref()) {
            Ok(target_type) => target_type,
            Err(err) => {
                log::error!("{}", err.to_string());
                let span_ref = ctx_with_span.span();
                span_ref.set_status(Status::Error {
                    description: "get_target_type failed".into(),
                });
                span_ref.end();
                return Err(ScrapeError::TypeError(err.to_string()));
            }
        };

        log::info!(
            "event=request type={} target={}",
            kind.to_string(),
            target.url
        );

        if let Some(metrics) = self
            .build_http_metrics(kind, target, ctx_with_span.clone())
            .await
        {
            let span_ref = ctx_with_span.span();
            span_ref.set_status(Status::Ok);
            log::info!(
                "event=metrics target={} job=zookoo {} {} {}",
                target.url,
                metrics.dns.to_logfmt(),
                metrics.http.to_logfmt(),
                metrics
                    .tls
                    .as_ref()
                    .map(|t| t.to_logfmt())
                    .unwrap_or_default()
            );

            self.export_metrics(
                kind,
                target.url.clone(),
                metrics,
                ctx_with_span,
            );
        } else {
            let span_ref = ctx_with_span.span();
            span_ref.set_status(Status::Error {
                description: std::borrow::Cow::Borrowed("probe failed"),
            });
        }

        Ok(())
    }
}

impl HttpScraper {
    async fn build_http_metrics(
        &self,
        kind: TargetType,
        target: &HttpTarget,
        ctx: Context,
    ) -> Option<HttpRequestMetrics> {
        let span_attr = vec![
            KeyValue::new("url", target.url.clone()),
            KeyValue::new("http_method", target.method.clone()),
            KeyValue::new(
                "expected_status_code",
                target.expected_status_code.clone().to_string(),
            ),
        ];
        let ctx_with_span = child_span_from_context("build_http_metrics", ctx.clone(), span_attr);

        let dns_metrics = match dns_lookup(&target.url, ctx_with_span.clone()).await {
            Ok(m) => m,
            Err(err) => {
                let span_ref = ctx_with_span.span();
                span_ref.set_status(Status::Error {
                    description: "dns lookup failed".into(),
                });
                log::error!("DNS lookup failed for url={} err={}", &target.url, err);
                return None;
            }
        };

        let tls_metrics = if kind == TargetType::HTTPS {
            match inspect_tls(&target.url, ctx_with_span.clone()).await {
                Ok(m) => Some(m),
                Err(err) => {
                    let span_ref = ctx_with_span.span();
                    span_ref.set_status(Status::Error {
                        description: "tls lookup failed".into(),
                    });
                    log::error!("TLS inspection failed for url={} err={}", &target.url, err);
                    Some(TlsMetrics::invalid())
                }
            }
        } else {
            None
        };

        let http_metrics = match http_request(&self.client, target, ctx_with_span.clone()).await {
            Ok(m) => m,
            Err(err) => {
                let span_ref = ctx_with_span.span();
                span_ref.set_status(Status::Error {
                    description: "fail to send request".into(),
                });
                log::error!("HTTP request failed for url={} err={}", &target.url, err);
                return None;
            }
        };

        let span_ref = ctx_with_span.span();
        span_ref.set_status(Status::Ok);

        Some(HttpRequestMetrics {
            dns: dns_metrics,
            http: http_metrics,
            tls: tls_metrics,
            labels: target.labels.clone(),
        })
    }

    fn export_metrics(&self, _kind: TargetType, target: String, metrics: HttpRequestMetrics, ctx: Context) {
        let span_attr = vec![KeyValue::new("url", target.clone())];
        let ctx_with_span = child_span_from_context("build_http_metrics", ctx.clone(), span_attr);

        metrics.export(&target);

        let span_ref = ctx_with_span.span();
        span_ref.set_status(Status::Ok);
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
