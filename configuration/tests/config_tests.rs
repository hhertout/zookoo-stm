use configuration::{ConfigParser, Parse};

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn test_parse_hcl_config_with_http_targets() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        // Test defaults
        assert_eq!(config.defaults.log_level, "info");
        assert_eq!(config.defaults.probe_zone, Some("FRA".to_string()));

        let location = config.defaults.probe_location.as_ref().unwrap();
        assert_eq!(location.latitude, 48.858370);
        assert_eq!(location.longitude, 2.29448);

        // Test self_monitoring
        let self_mon = config.defaults.self_monitoring.as_ref().unwrap();
        assert!(self_mon.enable);
        assert_eq!(self_mon.otel_endpoint, "https://otel-grpc.neryolab.com");
        assert_eq!(self_mon.pyroscope_endpoint, "https://otel-pyroscope.neryolab.com");
        assert_eq!(self_mon.service_name, "zookoo");
        assert_eq!(self_mon.env, "test");

        // Test probe.http configuration
        let probe = config.probe.as_ref().unwrap();
        let http_configs = &probe.http;

        assert_eq!(http_configs.len(), 2);

        let http_config =
            http_configs.get("api_monitoring").expect("api_monitoring probe should exist");
        assert_eq!(http_config.targets.as_ref().unwrap().len(), 3);

        // Test first target
        let target1 = &http_config.targets.as_ref().unwrap()[0];
        assert_eq!(target1.url, "https://example.com");
        let labels1 = target1.labels.as_ref().unwrap();
        assert_eq!(labels1.get("zone"), Some(&"eu-west-1".to_string()));
        assert_eq!(labels1.get("env"), Some(&"production".to_string()));

        // Test second target
        let target2 = &http_config.targets.as_ref().unwrap()[1];
        assert_eq!(target2.url, "https://httpbin.org/status/200");
        let labels2 = target2.labels.as_ref().unwrap();
        assert_eq!(labels2.get("zone"), Some(&"us-east-1".to_string()));
        assert_eq!(labels2.get("env"), Some(&"staging".to_string()));

        // Test third target
        let target3 = &http_config.targets.as_ref().unwrap()[2];
        assert_eq!(target3.url, "https://www.google.com");
        let labels3 = target3.labels.as_ref().unwrap();
        assert_eq!(labels3.get("zone"), Some(&"eu-west-1".to_string()));
        assert_eq!(labels3.get("env"), Some(&"production".to_string()));

        // Test exporter.otlp configuration
        let exporter = config.exporter.as_ref().unwrap();
        let otel = exporter.otel.get("main").expect("main exporter should exist");
        assert_eq!(otel.url, "https://otel-grpc.neryolab.com");
    }

    #[test]
    fn test_parse_hcl_discovery_file() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let discovery = config.discovery.as_ref().unwrap();
        let file_configs = &discovery.file;

        // The HCL format uses discovery.file "json_targets" which should be parsed correctly
        assert!(file_configs.contains_key("json_targets"));
        let file_config = file_configs.get("json_targets").unwrap();
        assert_eq!(file_config.path, vec!["/etc/zookoo/targets.json"]);
    }

    #[test]
    fn test_parse_hcl_defaults_structure() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        // Verify defaults are correctly parsed
        assert_eq!(config.defaults.log_level, "info");
        assert!(config.defaults.probe_location.is_some());
        assert!(config.defaults.probe_zone.is_some());
        assert!(config.defaults.self_monitoring.is_some());
    }

    #[test]
    fn test_parse_hcl_probe_http_targets_count() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let probe = config.probe.as_ref().expect("probe configuration should be present");

        let http_config =
            probe.http.get("api_monitoring").expect("api_monitoring probe should exist");
        // Should have exactly 3 targets as defined in the HCL file
        assert_eq!(http_config.targets.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_parse_hcl_exporter_otlp_endpoint() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let exporter =
            config.exporter.as_ref().expect("exporter configuration should be configured");

        let otel = exporter.otel.get("main").expect("main exporter should exist");
        assert_eq!(otel.url, "https://otel-grpc.neryolab.com");

        // Verify default values for optional fields
        assert!(!otel.tls_insecure);
        assert!(otel.auth.is_none());
        assert!(otel.cert_path.is_none());
    }

    #[test]
    fn test_parse_hcl_http_target_urls() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let probe = config.probe.as_ref().unwrap();
        let http_config =
            probe.http.get("api_monitoring").expect("api_monitoring probe should exist");

        let urls: Vec<&str> =
            http_config.targets.as_ref().unwrap().iter().map(|t| t.url.as_str()).collect();

        assert!(urls.contains(&"https://example.com"));
        assert!(urls.contains(&"https://httpbin.org/status/200"));
        assert!(urls.contains(&"https://www.google.com"));
    }

    #[test]
    fn test_parse_hcl_http_target_labels() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let probe = config.probe.as_ref().unwrap();
        let http_config =
            probe.http.get("api_monitoring").expect("api_monitoring probe should exist");

        // All targets should have labels
        for target in http_config.targets.as_ref().unwrap() {
            assert!(target.labels.is_some(), "Each target should have labels");
            let labels = target.labels.as_ref().unwrap();
            assert!(labels.contains_key("zone"));
            assert!(labels.contains_key("env"));
        }
    }

    #[test]
    fn test_parse_hcl_self_monitoring_configuration() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let self_mon =
            config.defaults.self_monitoring.as_ref().expect("self_monitoring should be configured");

        assert!(self_mon.enable);
        assert_eq!(self_mon.otel_endpoint, "https://otel-grpc.neryolab.com");
        assert_eq!(self_mon.pyroscope_endpoint, "https://otel-pyroscope.neryolab.com");
        assert_eq!(self_mon.service_name, "zookoo");
        assert_eq!(self_mon.env, "test");
        assert!(!self_mon.tls_ignore); // default value
    }

    #[test]
    fn test_parse_hcl_probe_location() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let location =
            config.defaults.probe_location.as_ref().expect("probe_location should be configured");

        // Paris coordinates
        assert!((location.latitude - 48.858370).abs() < 0.0001);
        assert!((location.longitude - 2.29448).abs() < 0.0001);
    }

    #[test]
    fn test_parse_hcl_http_targets_have_default_values() {
        let parser = ConfigParser;
        let config = parser
            .parse_from_file::<configuration::HCL>("tests/test_config.hcl")
            .expect("Failed to parse HCL config file");

        let probe = config.probe.as_ref().unwrap();
        let http_config =
            probe.http.get("api_monitoring").expect("api_monitoring probe should exist");

        // Verify that targets have default values for non-specified fields
        for target in http_config.targets.as_ref().unwrap() {
            // Default HTTP method should be GET
            assert_eq!(target.method, "GET");
            // Default expected status code should be 200
            assert_eq!(target.expected_status_code, 200);
            // Default timeout should be 15 seconds
            assert_eq!(target.timeout_sec, 15);
        }
    }
}
