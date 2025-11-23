#[cfg(test)]
mod tests {
    use crate::{ProbeType, ExporterRequest, ExporterConfigurationRequest};
    use std::collections::HashMap;

    #[test]
    fn test_probe_type_display() {
        assert_eq!(format!("{}", ProbeType::Http), "HTTP");
        assert_eq!(format!("{}", ProbeType::Icmp), "ICMP");
    }

    #[test]
    fn test_probe_type_equality() {
        assert_eq!(ProbeType::Http, ProbeType::Http);
        assert_eq!(ProbeType::Icmp, ProbeType::Icmp);
        assert_ne!(ProbeType::Http, ProbeType::Icmp);
    }

    #[test]
    fn test_probe_type_clone() {
        let probe1 = ProbeType::Http;
        let probe2 = probe1;
        assert_eq!(probe1, probe2);
    }

    #[test]
    fn test_exporter_request_creation() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("duration".to_string(), 100);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics: metrics.clone(),
        };

        assert_eq!(request.metrics.len(), 2);
        assert_eq!(request.metrics.get("up"), Some(&1));
        assert_eq!(request.metrics.get("duration"), Some(&100));
    }

    #[test]
    fn test_exporter_request_clone() {
        let mut metrics = HashMap::new();
        metrics.insert("test".to_string(), 42);

        let request1 = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        let request2 = request1.clone();
        assert_eq!(request1.metrics.get("test"), request2.metrics.get("test"));
    }

    #[test]
    fn test_exporter_request_empty_metrics() {
        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics: HashMap::new(),
        };

        assert_eq!(request.metrics.len(), 0);
        assert!(request.metrics.is_empty());
    }

    #[test]
    fn test_exporter_request_large_metrics() {
        let mut metrics = HashMap::new();
        for i in 0..1000 {
            metrics.insert(format!("metric_{}", i), i);
        }

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        assert_eq!(request.metrics.len(), 1000);
    }

    #[test]
    fn test_metrics_negative_values() {
        let mut metrics = HashMap::new();
        metrics.insert("negative".to_string(), -100);
        metrics.insert("positive".to_string(), 100);
        metrics.insert("zero".to_string(), 0);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        assert_eq!(request.metrics.get("negative"), Some(&-100));
        assert_eq!(request.metrics.get("positive"), Some(&100));
        assert_eq!(request.metrics.get("zero"), Some(&0));
    }

    #[test]
    fn test_probe_type_debug_format() {
        let http = ProbeType::Http;
        let icmp = ProbeType::Icmp;

        assert_eq!(format!("{:?}", http), "Http");
        assert_eq!(format!("{:?}", icmp), "Icmp");
    }

    #[test]
    fn test_exporter_request_debug_format() {
        let mut metrics = HashMap::new();
        metrics.insert("test".to_string(), 1);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        let debug_str = format!("{:?}", request);
        assert!(debug_str.contains("ExporterRequest"));
        assert!(debug_str.contains("test"));
    }
}
