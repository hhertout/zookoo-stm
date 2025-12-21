#[derive(Debug, Clone, Copy)]
pub(crate) enum ProbeType {
    Http,
    Icmp,
    Tcp,
}

impl From<ProbeType> for exporter::types::ProbeType {
    fn from(pt: ProbeType) -> exporter::types::ProbeType {
        match pt {
            ProbeType::Http => exporter::types::ProbeType::Http,
            ProbeType::Icmp => exporter::types::ProbeType::Icmp,
            ProbeType::Tcp => exporter::types::ProbeType::Tcp,
        }
    }
}

/// Convert probe::MetricData to exporter::MetricData
pub(crate) fn convert_metric_data(data: probe::MetricData) -> exporter::MetricData {
    exporter::MetricData { metrics: data.metrics, labels: data.labels }
}
