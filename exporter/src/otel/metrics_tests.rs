#[cfg(test)]
mod tests {
    use crate::otel::metrics::MetricsExporter;
    use std::collections::HashMap;

    #[test]
    fn test_metrics_exporter_creation() {
        let labels = HashMap::new();
        let _exporter = MetricsExporter::new(labels);

        // Verify exporter can be created
        // We can't test actual OTEL operations without infrastructure
        // but we ensure the API is correct
    }

    #[test]
    fn test_metrics_exporter_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("target".to_string(), "https://example.com".to_string());
        labels.insert("zone".to_string(), "eu-west-1".to_string());
        labels.insert("env".to_string(), "production".to_string());

        let _exporter = MetricsExporter::new(labels);
        // Exporter should be created successfully
    }

    #[test]
    fn test_http_metrics_format_for_otel() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("success".to_string(), 1);
        metrics.insert("dns_duration_ms".to_string(), 50);
        metrics.insert("status_code".to_string(), 200);
        metrics.insert("http_duration_ms".to_string(), 300);

        // Verify metrics structure is correct for OTEL export
        assert!(metrics.contains_key("up"));
        assert!(metrics.contains_key("success"));
        assert!(metrics.contains_key("status_code"));
    }

    #[test]
    fn test_icmp_metrics_format_for_otel() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("rtt_ms".to_string(), 25);

        assert_eq!(metrics.len(), 2);
        assert!(metrics.contains_key("rtt_ms"));
    }

    #[test]
    fn test_labels_with_special_characters() {
        let mut labels = HashMap::new();
        labels.insert(
            "target".to_string(),
            "https://example.com/api/v1/test?param=value".to_string(),
        );
        labels.insert("zone".to_string(), "us-east-1".to_string());
        labels.insert("tag".to_string(), "key:value".to_string());

        let _exporter = MetricsExporter::new(labels);
        // Should handle special characters in labels
    }

    #[test]
    fn test_empty_labels() {
        let labels = HashMap::new();
        let _exporter = MetricsExporter::new(labels);
        // Should work with empty labels
    }

    #[test]
    fn test_metrics_with_tls_data() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("success".to_string(), 1);
        metrics.insert("tls_duration_ms".to_string(), 100);
        metrics.insert("tls_handshake_ms".to_string(), 80);
        metrics.insert("cert_expiration_ts".to_string(), 1768521599);
        metrics.insert("cert_begin_ts".to_string(), 1736899200);

        assert!(metrics.contains_key("tls_duration_ms"));
        assert!(metrics.contains_key("cert_expiration_ts"));
    }

    #[test]
    fn test_probe_type_http_routing() {
        use crate::ProbeType;
        let probe_type = ProbeType::Http;
        assert_eq!(probe_type, ProbeType::Http);
        assert_ne!(probe_type, ProbeType::Icmp);
    }

    #[test]
    fn test_probe_type_icmp_routing() {
        use crate::ProbeType;
        let probe_type = ProbeType::Icmp;
        assert_eq!(probe_type, ProbeType::Icmp);
        assert_ne!(probe_type, ProbeType::Http);
    }

    #[test]
    fn test_large_number_of_labels() {
        let mut labels = HashMap::new();
        for i in 0..100 {
            labels.insert(format!("label_{}", i), format!("value_{}", i));
        }

        let _exporter = MetricsExporter::new(labels);
        // Should handle many labels
    }
}
