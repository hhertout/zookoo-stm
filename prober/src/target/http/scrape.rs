use futures::future::join_all;
use opentelemetry::trace::TraceContextExt;
use opentelemetry::{Context, KeyValue, global::ObjectSafeSpan, trace::Status};
use std::io::{Error, ErrorKind};

use crate::child_span_from_context;
use crate::target::ScrapeError;
use crate::{
    config::target::HttpTarget,
    get_tracer,
    metrics::{MetricExportable, Metrics, http_metrics::HttpRequestMetrics},
    target::{
        Scraping, TargetType,
        http::{
            dns::dns_lookup,
            request::http_request,
            tls::{TlsMetrics, inspect_tls},
        },
    },
    tracing_new_span,
};

#[derive(Clone)]
pub struct HttpScrapper {
    pub targets: Vec<HttpTarget>,
}

impl Scraping<HttpTarget> for HttpScrapper {
    fn new(targets: Vec<HttpTarget>) -> Self {
        HttpScrapper { targets }
    }

    async fn scrape(&self) -> Result<(), ScrapeError> {
        let mut span = tracing_new_span(get_tracer(), "scrape_target".to_string());
        span.set_attribute(KeyValue::new("type", "icmp".to_string()));
        let cx = Context::current_with_span(span);
        let guard = cx.clone().attach();

        let futures = self.targets.iter().map(|target| {
            let ctx = cx.clone();
            self.send_request(target, ctx)
        });

        drop(guard);
        let _ = join_all(futures).await;

        Ok(())
    }

    async fn send_request(&self, target: &HttpTarget, cx: Context) -> Result<(), ScrapeError> {
        let span_attr = vec![
            KeyValue::new("url", target.url.clone()),
            KeyValue::new("http_method", target.method.clone()),
            KeyValue::new(
                "expected_status_code",
                target.expected_status_code.clone().to_string(),
            ),
        ];
        let cx_with_span = child_span_from_context("send_request", cx.clone(), span_attr);

        let kind = match self.get_target_type(target.url.as_ref()) {
            Ok(target_type) => target_type,
            Err(err) => {
                log::error!("{}", err.to_string());
                let span_ref = cx_with_span.span();
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
            .build_http_metrics(kind, target, cx_with_span.clone())
            .await
        {
            let span_ref = cx_with_span.span();
            span_ref.set_status(Status::Ok);
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

            self.export_metrics(
                kind,
                target.url.clone(),
                Metrics::Http(metrics),
                cx_with_span,
            );
        } else {
            let span_ref = cx_with_span.span();
            span_ref.set_status(Status::Error {
                description: std::borrow::Cow::Borrowed("probe failed"),
            });
        }

        Ok(())
    }
}

impl HttpScrapper {
    async fn build_http_metrics(
        &self,
        kind: TargetType,
        target: &HttpTarget,
        cx: Context,
    ) -> Option<HttpRequestMetrics> {
        let span_attr = vec![
            KeyValue::new("url", target.url.clone()),
            KeyValue::new("http_method", target.method.clone()),
            KeyValue::new(
                "expected_status_code",
                target.expected_status_code.clone().to_string(),
            ),
        ];
        let cx_with_span = child_span_from_context("build_http_metrics", cx.clone(), span_attr);

        let dns_metrics = match dns_lookup(&target.url, cx_with_span.clone()).await {
            Ok(m) => m,
            Err(err) => {
                let span_ref = cx_with_span.span();
                span_ref.set_status(Status::Error {
                    description: "dns lookup failed".into(),
                });
                log::error!("DNS lookup failed for url={} err={}", &target.url, err);
                return None;
            }
        };

        let tls_metrics = if kind == TargetType::HTTPS {
            match inspect_tls(&target.url, cx_with_span.clone()).await {
                Ok(m) => Some(m),
                Err(err) => {
                    let span_ref = cx_with_span.span();
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

        let http_metrics = match http_request(target, cx_with_span.clone()).await {
            Ok(m) => m,
            Err(err) => {
                let span_ref = cx_with_span.span();
                span_ref.set_status(Status::Error {
                    description: "fail to send request".into(),
                });
                log::error!("HTTP request failed for url={} err={}", &target.url, err);
                return None;
            }
        };

        let span_ref = cx_with_span.span();
        span_ref.set_status(Status::Ok);

        Some(HttpRequestMetrics {
            dns: dns_metrics,
            http: http_metrics,
            tls: tls_metrics,
            labels: target.labels.clone(),
        })
    }

    fn export_metrics(&self, kind: TargetType, target: String, metrics: Metrics, cx: Context) {
        let span_attr = vec![KeyValue::new("url", target.clone())];
        let cx_with_span = child_span_from_context("build_http_metrics", cx.clone(), span_attr);

        match (kind, metrics) {
            (TargetType::HTTP | TargetType::HTTPS, Metrics::Http(m)) => m.export(&target),
            _ => {
                log::error!(
                    "wrong exporter type, got {}, expect https or https",
                    kind.to_string()
                )
            }
        };

        let span_ref = cx_with_span.span();
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
