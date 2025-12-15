use prost::Message;
use reqwest::Client;
use std::collections::HashMap;
use std::error::Error as StdError;
use std::time::{SystemTime, UNIX_EPOCH};

use configuration::model::exporter::AuthConfiguration;

/// Prometheus remote_write sample
#[derive(Clone, PartialEq, Message)]
pub struct Sample {
    #[prost(double, tag = "1")]
    pub value: f64,
    #[prost(int64, tag = "2")]
    pub timestamp: i64,
}

/// Prometheus remote_write label
#[derive(Clone, PartialEq, Message)]
pub struct Label {
    #[prost(string, tag = "1")]
    pub name: String,
    #[prost(string, tag = "2")]
    pub value: String,
}

/// Prometheus remote_write time series
#[derive(Clone, PartialEq, Message)]
pub struct TimeSeries {
    #[prost(message, repeated, tag = "1")]
    pub labels: Vec<Label>,
    #[prost(message, repeated, tag = "2")]
    pub samples: Vec<Sample>,
}

/// Prometheus remote_write request
#[derive(Clone, PartialEq, Message)]
pub struct WriteRequest {
    #[prost(message, repeated, tag = "1")]
    pub timeseries: Vec<TimeSeries>,
}

/// Configuration for Prometheus remote_write exporter
#[derive(Debug, Clone)]
pub struct PrometheusRemoteWriteConfig {
    pub url: String,
    pub auth: Option<AuthConfiguration>,
    pub job: String,
    pub instance: Option<String>,
    pub extra_labels: HashMap<String, String>,
}

/// Prometheus remote_write exporter
///
/// This exporter sends metrics to Prometheus-compatible endpoints that support
/// the remote_write API, such as:
/// - Prometheus itself (with remote_write receiver enabled)
/// - Grafana Alloy (prometheus.receive_http component)
/// - Grafana Mimir
/// - Thanos
/// - Victoria Metrics
///
/// The data is encoded in Prometheus protobuf format and compressed with Snappy.
pub struct PrometheusRemoteWrite {
    client: Client,
    config: PrometheusRemoteWriteConfig,
}

impl PrometheusRemoteWrite {
    /// Create a new Prometheus remote_write exporter
    pub fn new(config: PrometheusRemoteWriteConfig) -> Result<Self, Box<dyn StdError>> {
        let client = Client::builder().timeout(std::time::Duration::from_secs(30)).build()?;

        Ok(Self { client, config })
    }

    /// Push metrics to the remote_write endpoint
    ///
    /// # Arguments
    /// * `metric_name` - Name of the metric
    /// * `value` - Value of the metric
    /// * `labels` - Additional labels for the metric
    /// * `timestamp` - Optional timestamp (defaults to current time)
    pub async fn push_metric(
        &self,
        metric_name: &str,
        value: f64,
        labels: HashMap<String, String>,
        timestamp: Option<i64>,
    ) -> Result<(), Box<dyn StdError>> {
        let mut time_series_labels = vec![
            Label { name: "__name__".to_string(), value: metric_name.to_string() },
            Label { name: "job".to_string(), value: self.config.job.clone() },
        ];

        // Add instance if configured
        if let Some(instance) = &self.config.instance {
            time_series_labels
                .push(Label { name: "instance".to_string(), value: instance.clone() });
        }

        // Add extra labels from config
        for (key, val) in &self.config.extra_labels {
            time_series_labels.push(Label { name: key.clone(), value: val.clone() });
        }

        // Add metric-specific labels
        for (key, val) in labels {
            time_series_labels.push(Label { name: key, value: val });
        }

        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
        });

        let sample = Sample { value, timestamp: ts };

        let time_series = TimeSeries { labels: time_series_labels, samples: vec![sample] };

        let write_request = WriteRequest { timeseries: vec![time_series] };

        self.send_write_request(write_request).await
    }

    /// Push multiple metrics at once
    pub async fn push_metrics(
        &self,
        metrics: Vec<(String, f64, HashMap<String, String>)>,
        timestamp: Option<i64>,
    ) -> Result<(), Box<dyn StdError>> {
        let ts = timestamp.unwrap_or_else(|| {
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
        });

        let mut timeseries = Vec::new();

        for (metric_name, value, labels) in metrics {
            let mut time_series_labels =
                vec![Label { name: "__name__".to_string(), value: metric_name }];

            // Add metric-specific labels (which already contain job, instance, etc.)
            for (key, val) in labels {
                time_series_labels.push(Label { name: key, value: val });
            }

            let sample = Sample { value, timestamp: ts };

            timeseries.push(TimeSeries { labels: time_series_labels, samples: vec![sample] });
        }

        let write_request = WriteRequest { timeseries };

        self.send_write_request(write_request).await
    }

    /// Send the write request to the remote endpoint
    async fn send_write_request(
        &self,
        write_request: WriteRequest,
    ) -> Result<(), Box<dyn StdError>> {
        log::debug!(
            "Sending {} time series to {}",
            write_request.timeseries.len(),
            self.config.url
        );

        // Encode to protobuf
        let mut buf = Vec::new();
        write_request.encode(&mut buf)?;

        // Compress with Snappy
        let compressed = snap::raw::Encoder::new().compress_vec(&buf)?;

        log::debug!("Encoded {} bytes, compressed to {} bytes", buf.len(), compressed.len());

        // Build request
        let mut request = self
            .client
            .post(&self.config.url)
            .header("Content-Encoding", "snappy")
            .header("Content-Type", "application/x-protobuf")
            .header("X-Prometheus-Remote-Write-Version", "0.1.0")
            .body(compressed);

        // Add authentication if configured
        if let Some(auth) = &self.config.auth {
            if let (Some(username), Some(password)) = (&auth.username, &auth.password) {
                request = request.basic_auth(username, Some(password));
            } else if let Some(bearer) = &auth.bearer {
                request = request.bearer_auth(bearer);
            }
        }

        // Send request
        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            log::error!("Remote write failed with status {}: {}", status, body);
            return Err(format!("Remote write failed with status {}: {}", status, body).into());
        }

        log::debug!("Remote write successful, status: {}", response.status());
        Ok(())
    }
}
