//! HTTP Probe Implementation
//!
//! Implements the Probe trait for HTTP/HTTPS targets with unified timing.

use std::collections::HashMap;
use std::fmt::Display;
use std::sync::Arc;

use configuration::model::target::HttpTarget;
use futures::future::join_all;
use tokio::sync::Mutex;

use crate::{MetricData, Probe};

use super::client::{AuthConfig, HttpClient, HttpRequestConfig};
use super::metrics::HttpProbeMetrics;

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

/// HTTP Probe with unified phase timing
#[derive(Clone)]
pub struct HttpProbe {
    name: String,
    job: String,
    targets: Vec<HttpTarget>,
    client: Arc<HttpClient>,
    metrics: Arc<Mutex<Vec<MetricData>>>,
}

impl HttpProbe {
    /// Convert HttpTarget to HttpRequestConfig
    fn to_request_config(target: &HttpTarget) -> HttpRequestConfig {
        HttpRequestConfig {
            url: target.url.clone(),
            method: target.method.clone(),
            headers: target.headers.clone(),
            expected_status_code: target.expected_status_code,
            timeout_sec: target.timeout_sec,
            skip_tls: target.skip_tls,
            follow_redirect: target.follow_redirect,
            auth: target.auth.as_ref().map(|a| AuthConfig {
                username: a.username.clone(),
                password: a.password.clone(),
                bearer: a.bearer.clone(),
            }),
        }
    }

    /// Build MetricData from HttpProbeMetrics
    fn build_metric_data(
        target_url: &str,
        probe_metrics: &HttpProbeMetrics,
        target_labels: Option<HashMap<String, String>>,
    ) -> MetricData {
        let metrics_map = probe_metrics.to_metrics_map();

        MetricData::with_metrics(metrics_map)
            .with_labels(target_labels)
            .with_probe(crate::ProbeType::Http)
            .with_instance(target_url.to_string())
    }
}

impl Probe for HttpProbe {
    type Target = HttpTarget;

    fn init(name: String, job: String) -> Self {
        HttpProbe {
            name,
            job,
            targets: Vec::new(),
            client: Arc::new(HttpClient::new()),
            metrics: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_targets(&mut self, targets: Vec<Self::Target>) {
        self.targets = targets;
    }

    fn get_metrics(&self) -> impl std::future::Future<Output = Vec<MetricData>> + Send {
        let metrics = Arc::clone(&self.metrics);
        async move {
            let mut guard = metrics.lock().await;
            let result = guard.clone();
            guard.clear();
            result
        }
    }

    async fn scrape(&self) {
        let futures = self.targets.iter().map(|target| {
            let target = target.clone();
            let client = Arc::clone(&self.client);
            let metrics_store = Arc::clone(&self.metrics);

            async move {
                let kind = if target.url.starts_with("https") {
                    TargetType::HTTPS
                } else {
                    TargetType::HTTP
                };

                log::info!(
                    "event=request_start name={} job={} type={} target={}",
                    self.name,
                    self.job,
                    kind,
                    target.url
                );

                // Execute the probe with unified timing
                let config = Self::to_request_config(&target);
                let probe_metrics = client.execute(&config).await;

                log::info!(
                    "event=request_complete name={} job={} target={} duration={}ms",
                    self.name,
                    self.job,
                    target.url,
                    probe_metrics.total_duration.as_millis()
                );

                // Store metrics for export
                let metric_data =
                    Self::build_metric_data(&target.url, &probe_metrics, target.labels.clone());

                let mut guard = metrics_store.lock().await;
                guard.push(metric_data);
            }
        });

        let _ = join_all(futures).await;
    }
}
