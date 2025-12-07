use std::{collections::HashMap, fmt, sync::Arc};

use crate::Exporter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeType {
    Http,
    Icmp,
}

impl fmt::Display for ProbeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProbeType::Http => write!(f, "HTTP"),
            ProbeType::Icmp => write!(f, "ICMP"),
        }
    }
}

pub type ExportersMap = HashMap<String, Arc<dyn Exporter + Send + Sync>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExporterType {
    Otel,
    PrometheusRemoteWrite,
    Timescale,
}

impl fmt::Display for ExporterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExporterType::Otel => write!(f, "otel"),
            ExporterType::PrometheusRemoteWrite => write!(f, "prometheus_remote_write"),
            ExporterType::Timescale => write!(f, "timescale"),
        }
    }
}

impl ExporterType {
    pub fn iter() -> impl Iterator<Item = ExporterType> {
        [ExporterType::Otel, ExporterType::PrometheusRemoteWrite, ExporterType::Timescale]
            .iter()
            .copied()
    }
}
