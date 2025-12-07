#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use configuration::model::{
        Configuration, ExporterWrapper, defaults::Defaults,
        exporter::PrometheusRemoteWriteConfiguration,
    };

    use crate::prom::metrics::PrometheusRemoteWriteExporter;
    use crate::prom::remote_write::{PrometheusRemoteWrite, PrometheusRemoteWriteConfig};
    use crate::{Exporter, ExportersMap};

    /// Helper to create a minimal Configuration for testing
    fn create_test_config_with_prom(
        prom_configs: HashMap<String, PrometheusRemoteWriteConfiguration>,
    ) -> Configuration {
        Configuration {
            defaults: Defaults {
                log_level: "info".to_string(),
                job: "test-job".to_string(),
                service_name: "test-service".to_string(),
                probe_location: None,
                probe_zone: None,
                self_monitoring: None,
            },
            probe: None,
            exporter: Some(ExporterWrapper {
                otel: HashMap::new(),
                metrics: HashMap::new(),
                kafka: HashMap::new(),
                prometheus_remote_write: prom_configs,
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
            },
            probe: None,
            exporter: None,
            discovery: None,
        }
    }

    fn create_test_remote_write() -> Arc<PrometheusRemoteWrite> {
        let config = PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test".to_string(),
            instance: None,
            auth: None,
            extra_labels: HashMap::new(),
        };
        Arc::new(PrometheusRemoteWrite::new(config).expect("Failed to create remote write"))
    }

    // ==================== BUILD TESTS ====================

    #[test]
    fn test_build_with_no_exporters_configured() {
        let config = create_empty_config();
        let mut exporters: ExportersMap = HashMap::new();

        PrometheusRemoteWriteExporter::build(&config, &mut exporters);

        assert!(exporters.is_empty(), "No exporters should be created when none configured");
    }

    #[test]
    fn test_build_with_single_prometheus_exporter() {
        let mut prom_configs = HashMap::new();
        prom_configs.insert(
            "main".to_string(),
            PrometheusRemoteWriteConfiguration {
                url: "http://localhost:9090/api/v1/write".to_string(),
                job: "test-job".to_string(),
                instance: None,
                auth: None,
            },
        );

        let config = create_test_config_with_prom(prom_configs);
        let mut exporters: ExportersMap = HashMap::new();

        PrometheusRemoteWriteExporter::build(&config, &mut exporters);

        assert_eq!(exporters.len(), 1, "One exporter should be created");
        assert!(
            exporters.contains_key("exporter.prometheus_remote_write.main"),
            "Exporter key should match expected format"
        );
    }

    #[test]
    fn test_build_with_multiple_prometheus_exporters() {
        let mut prom_configs = HashMap::new();
        prom_configs.insert(
            "mimir".to_string(),
            PrometheusRemoteWriteConfiguration {
                url: "http://mimir:9090/api/v1/write".to_string(),
                job: "zookoo".to_string(),
                instance: Some("instance-1".to_string()),
                auth: None,
            },
        );
        prom_configs.insert(
            "victoria".to_string(),
            PrometheusRemoteWriteConfiguration {
                url: "http://victoria:8428/api/v1/write".to_string(),
                job: "zookoo".to_string(),
                instance: None,
                auth: None,
            },
        );

        let config = create_test_config_with_prom(prom_configs);
        let mut exporters: ExportersMap = HashMap::new();

        PrometheusRemoteWriteExporter::build(&config, &mut exporters);

        assert_eq!(exporters.len(), 2, "Two exporters should be created");
        assert!(exporters.contains_key("exporter.prometheus_remote_write.mimir"));
        assert!(exporters.contains_key("exporter.prometheus_remote_write.victoria"));
    }

    #[test]
    fn test_build_exporter_key_format() {
        let mut prom_configs = HashMap::new();
        prom_configs.insert(
            "my_custom_label".to_string(),
            PrometheusRemoteWriteConfiguration {
                url: "http://localhost:9090/api/v1/write".to_string(),
                job: "test".to_string(),
                instance: None,
                auth: None,
            },
        );

        let config = create_test_config_with_prom(prom_configs);
        let mut exporters: ExportersMap = HashMap::new();

        PrometheusRemoteWriteExporter::build(&config, &mut exporters);

        let expected_key = "exporter.prometheus_remote_write.my_custom_label";
        assert!(
            exporters.contains_key(expected_key),
            "Exporter key should be 'exporter.prometheus_remote_write.<label>'"
        );
    }

    // ==================== EXPORTER CREATION TESTS ====================

    #[test]
    fn test_prometheus_exporter_new_with_empty_labels() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();
        let exporter = PrometheusRemoteWriteExporter::new(labels, remote_write);

        // Should create successfully
        let _: &dyn Exporter = &exporter;
    }

    #[test]
    fn test_prometheus_exporter_new_with_labels() {
        let mut labels = HashMap::new();
        labels.insert("job".to_string(), "test-job".to_string());
        labels.insert("env".to_string(), "production".to_string());

        let remote_write = create_test_remote_write();
        let exporter = PrometheusRemoteWriteExporter::new(labels, remote_write);

        let _: &dyn Exporter = &exporter;
    }

    #[test]
    fn test_prometheus_exporter_with_prefix() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();

        let exporter = PrometheusRemoteWriteExporter::with_prefix(
            "custom_prefix".to_string(),
            labels,
            remote_write,
        );

        let _: &dyn Exporter = &exporter;
    }

    #[test]
    fn test_prometheus_exporter_prefix_auto_underscore() {
        let labels = HashMap::new();
        let remote_write = create_test_remote_write();

        // Prefix without trailing underscore should get one added
        let exporter = PrometheusRemoteWriteExporter::with_prefix(
            "myprefix".to_string(),
            labels.clone(),
            remote_write.clone(),
        );

        // We can't directly test the prefix, but we ensure it doesn't panic
        let _: &dyn Exporter = &exporter;

        // Prefix with trailing underscore should stay as-is
        let exporter2 = PrometheusRemoteWriteExporter::with_prefix(
            "myprefix_".to_string(),
            labels,
            remote_write,
        );
        let _: &dyn Exporter = &exporter2;
    }

    // ==================== REMOTE WRITE CONFIG TESTS ====================

    #[test]
    fn test_remote_write_config_creation() {
        let config = PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test-job".to_string(),
            instance: Some("test-instance".to_string()),
            auth: None,
            extra_labels: HashMap::new(),
        };

        let remote_write = PrometheusRemoteWrite::new(config);
        assert!(remote_write.is_ok(), "Remote write should be created successfully");
    }

    #[test]
    fn test_remote_write_config_with_extra_labels() {
        let mut extra_labels = HashMap::new();
        extra_labels.insert("env".to_string(), "production".to_string());
        extra_labels.insert("region".to_string(), "eu-west-1".to_string());

        let config = PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test-job".to_string(),
            instance: None,
            auth: None,
            extra_labels,
        };

        let remote_write = PrometheusRemoteWrite::new(config);
        assert!(remote_write.is_ok());
    }

    #[test]
    fn test_remote_write_config_with_auth() {
        use crate::config::AuthConfiguration;

        let config = PrometheusRemoteWriteConfig {
            url: "http://localhost:9090/api/v1/write".to_string(),
            job: "test-job".to_string(),
            instance: None,
            auth: Some(AuthConfiguration {
                username: Some("user".to_string()),
                password: Some("pass".to_string()),
                bearer: None,
            }),
            extra_labels: HashMap::new(),
        };

        let remote_write = PrometheusRemoteWrite::new(config);
        assert!(remote_write.is_ok());
    }
}
