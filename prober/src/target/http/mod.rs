use futures::future::join_all;
use opentelemetry::{Context, KeyValue, global::ObjectSafeSpan, trace::Status};
use std::io::{Error, ErrorKind};

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
    tracing_new_span, tracing_new_span_with_context,
};

use opentelemetry::trace::TraceContextExt;

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
        let span = tracing_new_span(get_tracer(), String::from("scrape"));
        let cx = Context::current_with_span(span);
        let _guard = cx.clone().attach();

        let futures = self.targets.iter().map(|target| {
            let ctx = cx.clone();
            self.send_request(target, ctx)
        });

        drop(_guard);
        let _ = join_all(futures).await;

        Ok(())
    }

    async fn send_request(&self, target: &HttpTarget, cx: Context) -> Result<(), Error> {
        let mut span =
            tracing_new_span_with_context(get_tracer(), String::from("send_request"), cx.clone());
        span.set_attribute(KeyValue::new("url", target.url.clone()));
        span.set_attribute(KeyValue::new("http_method", target.method.clone()));
        span.set_attribute(KeyValue::new(
            "expected_status_code",
            target.expected_status_code.clone().to_string(),
        ));
        let cx_with_span = cx.with_span(span);

        let kind = match self.get_target_type(target.url.as_ref()) {
            Ok(target_type) => target_type,
            Err(err) => {
                let span_ref = cx_with_span.span();
                span_ref.set_status(Status::Error {
                    description: "get_target_type failed".into(),
                });
                span_ref.end();
                return Err(err);
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
        let mut span = tracing_new_span_with_context(
            get_tracer(),
            String::from("build_http_metrics"),
            cx.clone(),
        );
        span.set_attribute(KeyValue::new("url", target.url.clone()));
        span.set_attribute(KeyValue::new("http_method", target.method.clone()));
        let cx_with_span = cx.with_span(span);

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
        let mut span =
            tracing_new_span_with_context(get_tracer(), String::from("export_metrics"), cx.clone());
        span.set_attribute(KeyValue::new("url", target.clone()));
        let cx_with_span = cx.with_span(span);

        match (kind, metrics) {
            (TargetType::HTTP | TargetType::HTTPS, Metrics::Http(m)) => m.export(&target),
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
