use std::{collections::HashMap, fmt::Display, sync::Arc};

use configuration::model::target::HttpTarget;
use futures::future::join_all;
use opentelemetry::{
    global::ObjectSafeSpan,
    trace::{Status, TraceContextExt},
};
use reqwest::Client;
use tokio::sync::Mutex;

use crate::{
    MetricData, Probe,
    http::{
        dns::dns_lookup,
        request::http_request,
        tls::{TlsMetrics, inspect_tls},
    },
    observability::get_empty_attributes,
};

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    HTTP,
    HTTPS,
}

impl Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::HTTP => write!(f, "HTTP"),
            TargetType::HTTPS => write!(f, "HTTPS"),
        }
    }
}

#[derive(Clone)]
pub struct HttpProbe {
    targets: Vec<HttpTarget>,
    client: Arc<Client>,
    metrics: Arc<Mutex<Vec<MetricData>>>,
}

impl HttpProbe {
    /// Build metrics map from HTTP probe results
    async fn build_http_metrics_map(
        &self,
        target_url: &str,
        dns_metrics: &crate::http::dns::DnsMetrics,
        http_metrics: &crate::http::request::HttpMetrics,
        tls_metrics: &Option<TlsMetrics>,
        target_labels: Option<HashMap<String, String>>,
    ) {
        let mut metrics_map = std::collections::HashMap::new();

        metrics_map.insert("up".to_string(), http_metrics.up as isize);
        metrics_map.insert("success".to_string(), http_metrics.success as isize);
        metrics_map
            .insert("dns_duration_ms".to_string(), dns_metrics.duration.as_millis() as isize);
        metrics_map.insert("status_code".to_string(), http_metrics.status_code as isize);
        metrics_map
            .insert("http_duration_ms".to_string(), http_metrics.duration.as_millis() as isize);

        if let Some(tls) = tls_metrics {
            metrics_map.insert("tls_duration_ms".to_string(), tls.duration.as_millis() as isize);
            metrics_map.insert(
                "tls_handshake_ms".to_string(),
                tls.handshake_duration.as_millis() as isize,
            );
            if let Some(exp) = tls.cert_expiration_date {
                metrics_map.insert("cert_expiration_ts".to_string(), exp as isize);
            }
            if let Some(begin) = tls.cert_begin_date {
                metrics_map.insert("cert_begin_ts".to_string(), begin as isize);
            }
        }

        let metric_data = MetricData::with_metrics(metrics_map)
            .with_labels(target_labels)
            .with_probe(crate::ProbeType::Http)
            .with_instance(target_url.to_string());

        let mut metrics = self.metrics.lock().await;
        metrics.push(metric_data);
    }
}

impl Probe for HttpProbe {
    type Target = HttpTarget;

    fn init() -> Self {
        crate::span!("init".to_string(), get_empty_attributes());

        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .build()
            .unwrap_or_else(|e| {
                log::error!("event=error msg=failed_to_build_http_client err={}", e);
                Client::new()
            });

        HttpProbe {
            targets: Vec::new(),
            client: Arc::new(client),
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_targets(&mut self, targets: Vec<Self::Target>) {
        crate::span!("set_targets".to_string(), get_empty_attributes());

        self.targets = targets;
    }

    fn get_metrics(&self) -> impl std::future::Future<Output = Vec<MetricData>> + Send {
        crate::span!("get_metrics".to_string(), get_empty_attributes());

        let metrics = Arc::clone(&self.metrics);
        async move {
            let mut guard = metrics.lock().await;
            let result = guard.clone();
            guard.clear(); // Clear metrics after reading
            result
        }
    }

    async fn scrape(&self) {
        let ctx = crate::span!("scrape_target".to_string(), get_empty_attributes());
        let guard = ctx.clone().attach();

        let futures = self.targets.iter().map(|target| {
            let target = target.clone();
            let ctx = ctx.clone();
            let client = Arc::clone(&self.client);
            async move {
                let mut attr = HashMap::new();
                attr.insert("url", target.url.clone());
                attr.insert("http_method", target.method.clone());
                attr.insert("expected_status_code", target.expected_status_code.to_string());
                let ctx_with_span = crate::child_span!(ctx, "scrape_http_target".to_string(), attr);

                // Determine target type
                let kind = if target.url.starts_with("https") {
                    TargetType::HTTPS
                } else {
                    TargetType::HTTP
                };

                log::info!("event=request type={} target={}", kind, target.url);

                // DNS lookup
                let dns_metrics = match dns_lookup(&target.url, ctx_with_span.clone()).await {
                    Ok(m) => m,
                    Err(err) => {
                        log::error!(
                            "event=error msg=dns_lookup_failed url={} err={}",
                            &target.url,
                            err
                        );
                        let span_ref = ctx_with_span.span();
                        span_ref
                            .set_status(Status::Error { description: "dns lookup failed".into() });
                        return;
                    }
                };

                // TLS inspection for HTTPS
                let tls_metrics = if kind == TargetType::HTTPS {
                    match inspect_tls(&target.url, ctx_with_span.clone()).await {
                        Ok(m) => Some(m),
                        Err(err) => {
                            log::error!(
                                "event=error msg=tls_inspection_failed url={} err={}",
                                &target.url,
                                err
                            );
                            Some(TlsMetrics::invalid())
                        }
                    }
                } else {
                    None
                };

                // HTTP request
                let http_metrics = match http_request(&client, &target, ctx_with_span.clone()).await
                {
                    Ok(m) => m,
                    Err(err) => {
                        log::error!(
                            "event=error msg=http_request_failed url={} err={}",
                            &target.url,
                            err
                        );
                        let span_ref = ctx_with_span.span();
                        span_ref.set_status(Status::Error {
                            description: "fail to send request".into(),
                        });
                        return;
                    }
                };

                let span_ref = ctx_with_span.span();
                span_ref.set_status(Status::Ok);

                log::info!(
                    "event=metrics target={} job=zookoo {} {} {}",
                    target.url,
                    dns_metrics.to_logfmt(),
                    http_metrics.to_logfmt(),
                    tls_metrics.as_ref().map(|t| t.to_logfmt()).unwrap_or_default()
                );

                // Build metrics for export (include target labels)
                self.build_http_metrics_map(
                    &target.url,
                    &dns_metrics,
                    &http_metrics,
                    &tls_metrics,
                    target.labels.clone(),
                )
                .await;
            }
        });

        drop(guard);
        let _ = join_all(futures).await;
    }
}
