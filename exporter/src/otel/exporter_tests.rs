#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use configuration::model::{
        Configuration, ExporterWrapper, defaults::Defaults, exporter::OtelGrpcExporterConfiguration,
    };

    use crate::otel::exporter::OtelExporter;
    use crate::{Exporter, ExportersMap, MetricData, ProbeType};

    /// Helper to create a minimal Configuration for testing
    fn create_test_config_with_otel(
        otel_configs: HashMap<String, OtelGrpcExporterConfiguration>,
    ) -> Configuration {
        Configuration {
            defaults: Defaults {
                log_level: "info".to_string(),
                job: "test-job".to_string(),
                service_name: "test-service".to_string(),
                probe_location: None,
                probe_zone: None,
                self_monitoring: None,
                metric_prefix: None,
            },
            probe: None,
            exporter: Some(ExporterWrapper {
                otel: otel_configs,
                prometheus_remote_write: HashMap::new(),
                timescale: HashMap::new(),
            }),
            discovery: None,
        }
    }

    fn create_empty_config() -> Configuration {
        Configuration {
            defaults: Defaults {
                log_level: "info".to_string(),
                job: "test-job".to_string(),
                service_name: "test-service".to_string(),
                probe_location: None,
                probe_zone: None,
                self_monitoring: None,
                metric_prefix: None,
            },
            probe: None,
            exporter: None,
            discovery: None,
        }
    }

    // ==================== BUILD TESTS ====================

    #[test]
    fn test_build_with_no_exporters_configured() {
        let config = create_empty_config();
        let mut exporters: ExportersMap = HashMap::new();

        OtelExporter::build(&config, &mut exporters);

        assert!(exporters.is_empty(), "No exporters should be created when none configured");
    }

    #[test]
    fn test_build_with_single_otel_exporter() {
        let mut otel_configs = HashMap::new();
        otel_configs.insert(
            "main".to_string(),
            OtelGrpcExporterConfiguration {
                url: "http://localhost:4317".to_string(),
                auth: None,
                cert_path: None,
                tls_insecure: false,
                metric_prefix: None,
            },
        );

        let config = create_test_config_with_otel(otel_configs);
        let mut exporters: ExportersMap = HashMap::new();

        OtelExporter::build(&config, &mut exporters);

        assert_eq!(exporters.len(), 1, "One exporter should be created");
        assert!(
            exporters.contains_key("exporter.otel.main"),
            "Exporter key should match expected format"
        );
    }

    #[test]
    fn test_build_with_multiple_otel_exporters() {
        let mut otel_configs = HashMap::new();
        otel_configs.insert(
            "primary".to_string(),
            OtelGrpcExporterConfiguration {
                url: "http://localhost:4317".to_string(),
                auth: None,
                cert_path: None,
                tls_insecure: false,
                metric_prefix: None,
            },
        );
        otel_configs.insert(
            "secondary".to_string(),
            OtelGrpcExporterConfiguration {
                url: "http://localhost:4318".to_string(),
                auth: None,
                cert_path: None,
                tls_insecure: true,
                metric_prefix: None,
            },
        );

        let config = create_test_config_with_otel(otel_configs);
        let mut exporters: ExportersMap = HashMap::new();

        OtelExporter::build(&config, &mut exporters);

        assert_eq!(exporters.len(), 2, "Two exporters should be created");
        assert!(exporters.contains_key("exporter.otel.primary"));
        assert!(exporters.contains_key("exporter.otel.secondary"));
    }

    #[test]
    fn test_build_exporter_key_format() {
        let mut otel_configs = HashMap::new();
        otel_configs.insert(
            "my_custom_label".to_string(),
            OtelGrpcExporterConfiguration {
                url: "http://localhost:4317".to_string(),
                auth: None,
                cert_path: None,
                tls_insecure: false,
                metric_prefix: None,
            },
        );

        let config = create_test_config_with_otel(otel_configs);
        let mut exporters: ExportersMap = HashMap::new();

        OtelExporter::build(&config, &mut exporters);

        let expected_key = "exporter.otel.my_custom_label";
        assert!(
            exporters.contains_key(expected_key),
            "Exporter key should be 'exporter.otel.<label>'"
        );
    }

    // ==================== OTEL EXPORTER CREATION TESTS ====================

    #[test]
    fn test_otel_exporter_new_with_empty_labels() {
        let labels = HashMap::new();
        let exporter = OtelExporter::new(labels, None);

        // Should create successfully
        let _: &dyn Exporter = &exporter;
    }

    #[test]
    fn test_otel_exporter_new_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("job".to_string(), "test-job".to_string());
        labels.insert("env".to_string(), "production".to_string());
        labels.insert("zone".to_string(), "eu-west-1".to_string());

        let exporter = OtelExporter::new(labels, None);

        // Should create successfully with labels
        let _: &dyn Exporter = &exporter;
    }

    // ==================== EXPORT METHOD TESTS ====================

    #[test]
    fn test_export_http_metrics_extracts_correct_values() {
        let labels = HashMap::new();
        let exporter = OtelExporter::new(labels, None);

        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("success".to_string(), 1);
        metrics.insert("status_code".to_string(), 200);
        metrics.insert("dns_duration_ms".to_string(), 50);
        metrics.insert("tcp_connect_duration_ms".to_string(), 30);
        metrics.insert("time_to_first_byte_ms".to_string(), 100);
        metrics.insert("content_transfer_duration_ms".to_string(), 150);
        metrics.insert("http_duration_ms".to_string(), 330);

        let mut target_labels = HashMap::new();
        target_labels.insert("target".to_string(), "https://example.com".to_string());

        let metric_data = MetricData::with_metrics(metrics).with_labels(target_labels);

        // This should not panic - we can't verify OTEL calls without infrastructure
        exporter.export(ProbeType::Http, metric_data);
    }

    #[test]
    fn test_export_http_metrics_with_optional_tls_fields() {
        let labels = HashMap::new();
        let exporter = OtelExporter::new(labels, None);

        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("success".to_string(), 1);
        metrics.insert("status_code".to_string(), 200);
        metrics.insert("http_duration_ms".to_string(), 500);
        metrics.insert("tls_duration_ms".to_string(), 100);
        metrics.insert("tls_handshake_ms".to_string(), 80);
        metrics.insert("cert_expiration_ts".to_string(), 1735689600);
        metrics.insert("cert_begin_ts".to_string(), 1704067200);

        let metric_data = MetricData::with_metrics(metrics);

        // Should handle optional TLS fields without panicking
        exporter.export(ProbeType::Http, metric_data);
    }

    #[test]
    fn test_export_http_metrics_with_missing_optional_fields() {
        let labels = HashMap::new();
        let exporter = OtelExporter::new(labels, None);

        // Minimal metrics - only required fields
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 0);

        let metric_data = MetricData::with_metrics(metrics);

        // Should handle missing fields gracefully (using defaults)
        exporter.export(ProbeType::Http, metric_data);
    }

    #[test]
    fn test_export_icmp_metrics() {
        let labels = HashMap::new();
        let exporter = OtelExporter::new(labels, None);

        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("rtt_ms".to_string(), 25);

        let mut target_labels = HashMap::new();
        target_labels.insert("target".to_string(), "8.8.8.8".to_string());

        let metric_data = MetricData::with_metrics(metrics).with_labels(target_labels);

        // Should not panic
        exporter.export(ProbeType::Icmp, metric_data);
    }

    #[test]
    fn test_export_icmp_metrics_with_missing_rtt() {
        let labels = HashMap::new();
        let exporter = OtelExporter::new(labels, None);

        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 0);
        // rtt_ms is missing (timeout case)

        let metric_data = MetricData::with_metrics(metrics);

        // Should handle missing rtt gracefully
        exporter.export(ProbeType::Icmp, metric_data);
    }

    // ==================== METRIC DATA TESTS ====================

    #[test]
    fn test_metric_data_with_instance() {
        let metrics = HashMap::new();
        let metric_data =
            MetricData::with_metrics(metrics).with_instance("my-instance".to_string());

        assert_eq!(metric_data.labels.get("instance"), Some(&"my-instance".to_string()));
    }

    #[test]
    fn test_metric_data_with_probe_type() {
        let metrics = HashMap::new();
        let metric_data = MetricData::with_metrics(metrics).with_probe(ProbeType::Http);

        assert_eq!(metric_data.labels.get("probe"), Some(&"HTTP".to_string()));
    }

    #[test]
    fn test_metric_data_chaining() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);

        let mut labels = HashMap::new();
        labels.insert("custom".to_string(), "value".to_string());

        let metric_data = MetricData::with_metrics(metrics)
            .with_labels(labels)
            .with_instance("test-instance".to_string())
            .with_probe(ProbeType::Icmp);

        assert_eq!(metric_data.metrics.get("up"), Some(&1));
        assert_eq!(metric_data.labels.get("custom"), Some(&"value".to_string()));
        assert_eq!(metric_data.labels.get("instance"), Some(&"test-instance".to_string()));
        assert_eq!(metric_data.labels.get("probe"), Some(&"ICMP".to_string()));
    }
}
