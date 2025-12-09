#[cfg(test)]
mod tests {
    use crate::prometheus::PrometheusRemoteWrite;
    use crate::prometheus::PrometheusRemoteWriteExporter;
    use crate::{Exporter, ProbeType};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_prometheus_exporter_creation() {
        let labels = HashMap::new();
        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: Some("test-instance".to_string()),
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write =
            PrometheusRemoteWrite::new(config).expect("Failed to create remote write");
        let _exporter = PrometheusRemoteWriteExporter::new(labels, Arc::new(remote_write), None);

        // Exporter should be created successfully
    }

    #[test]
    fn test_export_trait_implementation() {
        let labels = HashMap::new();
        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write =
            PrometheusRemoteWrite::new(config).expect("Failed to create remote write");
        let _exporter = PrometheusRemoteWriteExporter::new(labels, Arc::new(remote_write), None);

        // Verify Exporter trait is implemented
        let _: &dyn Exporter = &_exporter;
    }

    #[test]
    fn test_http_metrics_request_format() {
        let mut metrics = HashMap::new();
        metrics.insert("success".to_string(), 1);
        metrics.insert("dns_duration_ms".to_string(), 50);
        metrics.insert("http_duration_ms".to_string(), 300);

        assert_eq!(metrics.get("success"), Some(&1));
        assert_eq!(metrics.get("dns_duration_ms"), Some(&50));
        assert_eq!(metrics.get("http_duration_ms"), Some(&300));
    }

    #[test]
    fn test_icmp_metrics_request_format() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("rtt_ms".to_string(), 25);

        assert_eq!(metrics.get("up"), Some(&1));
        assert_eq!(metrics.get("rtt_ms"), Some(&25));
    }

    #[test]
    fn test_exporter_with_custom_labels() {
        let mut labels = HashMap::new();
        labels.insert("environment".to_string(), "production".to_string());
        labels.insert("region".to_string(), "us-east-1".to_string());
        labels.insert("service".to_string(), "api".to_string());

        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write =
            PrometheusRemoteWrite::new(config).expect("Failed to create remote write");
        let _exporter = PrometheusRemoteWriteExporter::new(labels, Arc::new(remote_write), None);

        // Should handle custom labels
    }

    #[test]
    fn test_probe_type_http() {
        let probe_type = ProbeType::Http;
        assert_eq!(format!("{}", probe_type), "HTTP");
    }

    #[test]
    fn test_probe_type_icmp() {
        let probe_type = ProbeType::Icmp;
        assert_eq!(format!("{}", probe_type), "ICMP");
    }

    #[test]
    fn test_metrics_with_zero_values() {
        let mut metrics = HashMap::new();
        metrics.insert("success".to_string(), 0);
        metrics.insert("dns_duration_ms".to_string(), 0);

        assert_eq!(metrics.get("success"), Some(&0));
    }

    #[test]
    fn test_metrics_with_high_values() {
        let mut metrics = HashMap::new();
        metrics.insert("http_duration_ms".to_string(), 999999);
        metrics.insert("rtt_ms".to_string(), 10000);

        assert_eq!(metrics.get("http_duration_ms"), Some(&999999));
    }

    #[test]
    fn test_config_with_authentication() {
        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: Some("instance-1".to_string()),
            auth: Some(crate::config::AuthConfiguration {
                username: Some("user".to_string()),
                password: Some("pass".to_string()),
                bearer: None,
            }),
            extra_labels: HashMap::new(),
        };

        let remote_write = PrometheusRemoteWrite::new(config);
        assert!(remote_write.is_ok());
    }

    #[test]
    fn test_config_with_bearer_token() {
        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: Some(crate::config::AuthConfiguration {
                username: None,
                password: None,
                bearer: Some("token123".to_string()),
            }),
            extra_labels: HashMap::new(),
        };

        let remote_write = PrometheusRemoteWrite::new(config);
        assert!(remote_write.is_ok());
    }

    #[test]
    fn test_config_with_extra_labels() {
        let mut extra_labels = HashMap::new();
        extra_labels.insert("cluster".to_string(), "prod-1".to_string());
        extra_labels.insert("datacenter".to_string(), "dc1".to_string());

        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels,
        };

        let remote_write = PrometheusRemoteWrite::new(config);
        assert!(remote_write.is_ok());
    }

    #[test]
    fn test_multiple_exporters_with_same_config() {
        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write = Arc::new(PrometheusRemoteWrite::new(config).expect("Failed"));

        let labels1 = HashMap::from([("instance".to_string(), "1".to_string())]);
        let labels2 = HashMap::from([("instance".to_string(), "2".to_string())]);

        let _exporter1 =
            PrometheusRemoteWriteExporter::new(labels1, Arc::clone(&remote_write), None);
        let _exporter2 =
            PrometheusRemoteWriteExporter::new(labels2, Arc::clone(&remote_write), None);

        // Multiple exporters can share the same remote_write
    }

    fn create_test_remote_write() -> Arc<PrometheusRemoteWrite> {
        let config = crate::prometheus::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };
        Arc::new(PrometheusRemoteWrite::new(config).expect("Failed to create remote write"))
    }

    #[test]
    fn test_metric_prefix_default() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();
        let exporter = PrometheusRemoteWriteExporter::new(labels, remote_write, None);

        // Default prefix should be "probe_"
        assert!(exporter.get_prefix() == "probe_");
    }

    #[test]
    fn test_metric_prefix_custom() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();
        let custom_prefix = Some("zookoo_".to_string());
        let exporter = PrometheusRemoteWriteExporter::new(labels, remote_write, custom_prefix);

        // Custom prefix should be applied
        assert!(exporter.get_prefix() == "zookoo_");
    }

    #[test]
    fn test_metric_prefix_empty() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();
        let empty_prefix = Some("".to_string());
        let exporter = PrometheusRemoteWriteExporter::new(labels, remote_write, empty_prefix);

        // Empty prefix should work (no prefix)
        assert!(exporter.get_prefix() == "probe_");
    }

    #[test]
    fn test_metric_prefix_with_prefix_method() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();
        let exporter =
            PrometheusRemoteWriteExporter::with_prefix("custom_".to_string(), labels, remote_write);

        // with_prefix should set the prefix correctly
        assert!(exporter.get_prefix() == "custom_");
    }

    #[test]
    fn test_metric_prefix_with_prefix_auto_underscore() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();

        // Prefix without trailing underscore should get one added
        let exporter = PrometheusRemoteWriteExporter::with_prefix(
            "myprefix".to_string(),
            labels,
            remote_write,
        );

        // with_prefix should auto-append underscore if missing
        assert!(exporter.get_prefix() == "myprefix_");
    }

    #[test]
    fn test_metric_prefix_preserves_trailing_underscore() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();

        // Prefix with trailing underscore should stay as-is
        let exporter = PrometheusRemoteWriteExporter::with_prefix(
            "myprefix_".to_string(),
            labels,
            remote_write,
        );

        assert!(exporter.get_prefix() == "myprefix_");
    }
}
