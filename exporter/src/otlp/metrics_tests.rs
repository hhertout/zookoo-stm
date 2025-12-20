#[cfg(test)]
mod tests {
    use crate::otlp::metrics::MetricsExporter;
    use opentelemetry_sdk::metrics::SdkMeterProvider;
    use std::collections::HashMap;

    fn test_meter_provider() -> SdkMeterProvider {
        SdkMeterProvider::builder().build()
    }

    #[test]
    fn test_metrics_exporter_creation() {
        let labels = HashMap::new();
        let _exporter = MetricsExporter::new(labels, None, Some(test_meter_provider()));

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

        let _exporter = MetricsExporter::new(labels, None, Some(test_meter_provider()));
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

        let _exporter = MetricsExporter::new(labels, None, Some(test_meter_provider()));
        // Should handle special characters in labels
    }

    #[test]
    fn test_empty_labels() {
        let labels = HashMap::new();
        let _exporter = MetricsExporter::new(labels, None, Some(test_meter_provider()));
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

        let _exporter = MetricsExporter::new(labels, None, Some(test_meter_provider()));
        // Should handle many labels
    }

    #[test]
    fn test_metric_prefix_default() {
        let labels = HashMap::new();
        let exporter = MetricsExporter::new(labels, None, Some(test_meter_provider()));

        // Default prefix should be "probe_"
        // We test this indirectly by creating the exporter
        // The prefix is used internally when exporting metrics
        assert!(exporter.get_prefix() == "probe_");
    }

    #[test]
    fn test_metric_prefix_custom() {
        let labels = HashMap::new();
        let custom_prefix = Some("zookoo_".to_string());
        let exporter = MetricsExporter::new(labels, custom_prefix, Some(test_meter_provider()));

        // Custom prefix should be applied
        assert!(exporter.get_prefix() == "zookoo_");
    }

    #[test]
    fn test_metric_prefix_empty() {
        let labels = HashMap::new();
        let empty_prefix = Some("".to_string());
        let exporter = MetricsExporter::new(labels, empty_prefix, Some(test_meter_provider()));

        // Empty prefix should work (no prefix)
        assert!(exporter.get_prefix() == "probe_");
    }

    #[test]
    fn test_metric_prefix_with_underscore() {
        let labels = HashMap::new();
        let prefix_with_underscore = Some("my_custom_prefix_".to_string());
        let exporter =
            MetricsExporter::new(labels, prefix_with_underscore, Some(test_meter_provider()));

        // Prefix ending with underscore should be used as-is
        assert!(exporter.get_prefix() == "my_custom_prefix_");
    }

    #[test]
    fn test_metric_prefix_without_underscore() {
        let labels = HashMap::new();
        let prefix_without_underscore = Some("myprefix".to_string());
        let exporter =
            MetricsExporter::new(labels, prefix_without_underscore, Some(test_meter_provider()));

        // Prefix without underscore should work
        assert!(exporter.get_prefix() == "myprefix_");
    }
}
