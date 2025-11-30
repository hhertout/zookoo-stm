#[derive(Debug, Clone, Copy)]
pub(crate) enum ProbeType {
    Http,
    Icmp,
}

impl From<ProbeType> for exporter::ProbeType {
    fn from(pt: ProbeType) -> exporter::ProbeType {
        match pt {
            ProbeType::Http => exporter::ProbeType::Http,
            ProbeType::Icmp => exporter::ProbeType::Icmp,
        }
    }
}

/// Convert probe::MetricData to exporter::MetricData
pub(crate) fn convert_metric_data(data: probe::MetricData) -> exporter::MetricData {
    exporter::MetricData { metrics: data.metrics, labels: data.labels }
}
