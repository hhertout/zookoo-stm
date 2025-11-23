#[cfg(test)]
mod tests {
    use crate::prom::PrometheusRemoteWriteExporter;
    use crate::prom::PrometheusRemoteWrite;
    use crate::{Export, ExporterRequest, ExporterConfigurationRequest, ProbeType};
    use std::collections::HashMap;
    use std::sync::Arc;

    #[test]
    fn test_prometheus_exporter_creation() {
        let labels = HashMap::new();
        let config = crate::prom::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: Some("test-instance".to_string()),
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write = PrometheusRemoteWrite::new(config).expect("Failed to create remote write");
        let _exporter = PrometheusRemoteWriteExporter::new(labels, Arc::new(remote_write));
        
        // Exporter should be created successfully
    }

    #[test]
    fn test_export_trait_implementation() {
        let labels = HashMap::new();
        let config = crate::prom::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write = PrometheusRemoteWrite::new(config).expect("Failed to create remote write");
        let _exporter = PrometheusRemoteWriteExporter::new(labels, Arc::new(remote_write));
        
        // Verify Export trait is implemented
        let _: &dyn Export = &_exporter;
    }

    #[test]
    fn test_http_metrics_request_format() {
        let mut metrics = HashMap::new();
        metrics.insert("success".to_string(), 1);
        metrics.insert("dns_duration_ms".to_string(), 50);
        metrics.insert("http_duration_ms".to_string(), 300);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics: metrics.clone(),
        };

        assert_eq!(request.metrics.get("success"), Some(&1));
        assert_eq!(request.metrics.get("dns_duration_ms"), Some(&50));
        assert_eq!(request.metrics.get("http_duration_ms"), Some(&300));
    }

    #[test]
    fn test_icmp_metrics_request_format() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("rtt_ms".to_string(), 25);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        assert_eq!(request.metrics.get("up"), Some(&1));
        assert_eq!(request.metrics.get("rtt_ms"), Some(&25));
    }

    #[test]
    fn test_exporter_with_custom_labels() {
        let mut labels = HashMap::new();
        labels.insert("environment".to_string(), "production".to_string());
        labels.insert("region".to_string(), "us-east-1".to_string());
        labels.insert("service".to_string(), "api".to_string());

        let config = crate::prom::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write = PrometheusRemoteWrite::new(config).expect("Failed to create remote write");
        let _exporter = PrometheusRemoteWriteExporter::new(labels, Arc::new(remote_write));
        
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

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        assert_eq!(request.metrics.get("success"), Some(&0));
    }

    #[test]
    fn test_metrics_with_high_values() {
        let mut metrics = HashMap::new();
        metrics.insert("http_duration_ms".to_string(), 999999);
        metrics.insert("rtt_ms".to_string(), 10000);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        assert_eq!(request.metrics.get("http_duration_ms"), Some(&999999));
    }

    #[test]
    fn test_config_with_authentication() {
        let config = crate::prom::PrometheusRemoteWriteConfig {
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
        let config = crate::prom::PrometheusRemoteWriteConfig {
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

        let config = crate::prom::PrometheusRemoteWriteConfig {
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
        let config = crate::prom::PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write = Arc::new(PrometheusRemoteWrite::new(config).expect("Failed"));

        let labels1 = HashMap::from([("instance".to_string(), "1".to_string())]);
        let labels2 = HashMap::from([("instance".to_string(), "2".to_string())]);

        let _exporter1 = PrometheusRemoteWriteExporter::new(labels1, Arc::clone(&remote_write));
        let _exporter2 = PrometheusRemoteWriteExporter::new(labels2, Arc::clone(&remote_write));

        // Multiple exporters can share the same remote_write
    }
}
