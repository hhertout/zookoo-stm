pub mod http;
pub mod icmp;

pub mod observability;

use std::{collections::HashMap, fmt::Display};

// Re-export probes for easy access
pub use http::HttpProbe;
pub use icmp::IcmpProbe;

#[derive(Debug)]
pub enum ScrapeError {
    TypeError(String),
    InvalidInput(String),
    LookupFailed,
    NetworkError(String),
}

impl std::fmt::Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScrapeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            ScrapeError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ScrapeError::LookupFailed => write!(f, "DNS lookup failed"),
            ScrapeError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for ScrapeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeType {
    Http,
    Icmp,
}

impl Display for ProbeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProbeType::Http => write!(f, "http"),
            ProbeType::Icmp => write!(f, "icmp"),
        }
    }
}

/// Metric data containing both numeric metrics and string labels from targets
#[derive(Debug, Clone)]
pub struct MetricData {
    /// Numeric metric values (e.g., duration_ms, status_code)
    pub metrics: HashMap<String, isize>,
    /// String labels from the target configuration (e.g., service, env)
    pub labels: HashMap<String, String>,
}

impl MetricData {
    pub fn new() -> Self {
        MetricData { metrics: HashMap::new(), labels: HashMap::new() }
    }

    pub fn with_metrics(metrics: HashMap<String, isize>) -> Self {
        MetricData { metrics, labels: HashMap::new() }
    }

    pub fn with_labels(mut self, labels: Option<HashMap<String, String>>) -> Self {
        if let Some(l) = labels {
            self.labels = l;
        }
        self
    }

    pub fn with_instance(mut self, instance: String) -> Self {
        self.labels.insert("instance".to_string(), instance);
        self
    }

    pub fn with_probe(mut self, probe: crate::ProbeType) -> Self {
        self.labels.insert("probe".to_string(), probe.to_string());
        self
    }
}

impl Default for MetricData {
    fn default() -> Self {
        Self::new()
    }
}

/// Probe trait: manage targets and forward results to exporters or other components.
pub trait Probe {
    type Target: Clone + std::fmt::Debug + Send + Sync + 'static;
    fn init() -> Self;

    /// Set or update the target data for this probe.
    fn set_targets(&mut self, data: Vec<Self::Target>);

    // Perform a scrape operation and return a Future.
    fn scrape(&self) -> impl std::future::Future<Output = ()> + Send;

    fn get_metrics(&self) -> impl std::future::Future<Output = Vec<MetricData>> + Send;
}
