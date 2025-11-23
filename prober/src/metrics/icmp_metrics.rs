use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::metrics::MetricExportable;

pub struct IcmpRequestMetrics {
    pub up: u8,
    pub duration: Duration,
    pub labels: Option<Arc<HashMap<String, String>>>,
}

impl MetricExportable for IcmpRequestMetrics {
    fn export(&self, target: &str) {
        let mut labels: HashMap<String, String> = HashMap::new();
        labels.insert(String::from("target"), target.to_string());

        if let Some(l) = &self.labels {
            labels.extend(l.as_ref().iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        let exporter = exporter::otel::metrics::MetricsExporter::new(labels);
        exporter.export_icmp_metrics(self.up, self.duration.as_millis());
    }
}
