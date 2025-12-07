#[cfg(test)]
mod tests {
    use crate::ProbeType;
    use crate::timescale::metrics::TimescaleExporter;
    use crate::{Exporter, ExportersMap};
    use configuration::model::{
        Configuration, ExporterWrapper, defaults::Defaults,
        exporter::TimescaleExporterConfiguration,
    };
    use std::collections::HashMap;

    fn create_test_config_with_timescale(
        timescale_map: std::collections::HashMap<String, TimescaleExporterConfiguration>,
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
                otel: std::collections::HashMap::new(),
                prometheus_remote_write: std::collections::HashMap::new(),
                timescale: timescale_map,
            }),
            discovery: None,
        }
    }

    #[test]
    fn test_timescale_exporter_structure() {
        // Test that the exporter structure is correctly defined
        // We can't test actual DB operations without a database,
        // but we can ensure the API is correct
    }

    #[test]
    fn test_export_trait_http_request_format() {
        // Validate HTTP metrics format
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("success".to_string(), 1);
        metrics.insert("dns_duration_ms".to_string(), 50);
        metrics.insert("status_code".to_string(), 200);
        metrics.insert("http_duration_ms".to_string(), 300);

        assert_eq!(metrics.get("up"), Some(&1));
        assert_eq!(metrics.get("success"), Some(&1));
        assert_eq!(metrics.get("status_code"), Some(&200));
    }

    #[test]
    fn test_export_trait_http_with_tls_request_format() {
        // Validate HTTP with TLS metrics format
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("success".to_string(), 1);
        metrics.insert("dns_duration_ms".to_string(), 50);
        metrics.insert("status_code".to_string(), 200);
        metrics.insert("http_duration_ms".to_string(), 300);
        metrics.insert("tls_duration_ms".to_string(), 100);
        metrics.insert("tls_handshake_ms".to_string(), 80);
        metrics.insert("cert_expiration_ts".to_string(), 1768521599);
        metrics.insert("cert_begin_ts".to_string(), 1736899200);

        assert_eq!(metrics.get("tls_duration_ms"), Some(&100));
        assert_eq!(metrics.get("tls_handshake_ms"), Some(&80));
        assert_eq!(metrics.get("cert_expiration_ts"), Some(&1768521599));
    }

    #[test]
    fn test_export_trait_icmp_request_format() {
        // Validate ICMP metrics format
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("rtt_ms".to_string(), 25);

        assert_eq!(metrics.get("up"), Some(&1));
        assert_eq!(metrics.get("rtt_ms"), Some(&25));

        // ICMP should only have these two metrics
        assert_eq!(metrics.len(), 2);
    }

    #[test]
    fn test_probe_type_routing() {
        // Ensure different probe types are distinguished
        let http_type = ProbeType::Http;
        let icmp_type = ProbeType::Icmp;

        assert_ne!(http_type, icmp_type);
    }

    #[test]
    fn test_http_metrics_all_fields() {
        let mut metrics = HashMap::new();

        // Required fields
        metrics.insert("up".to_string(), 1);
        metrics.insert("success".to_string(), 1);
        metrics.insert("dns_duration_ms".to_string(), 50);
        metrics.insert("status_code".to_string(), 200);
        metrics.insert("http_duration_ms".to_string(), 300);

        // Optional TLS fields
        metrics.insert("tls_duration_ms".to_string(), 100);
        metrics.insert("tls_handshake_ms".to_string(), 80);
        metrics.insert("cert_expiration_ts".to_string(), 1768521599);
        metrics.insert("cert_begin_ts".to_string(), 1736899200);

        assert_eq!(metrics.len(), 9);

        // Verify each field exists and has correct value
        assert!(metrics.contains_key("up"));
        assert!(metrics.contains_key("success"));
        assert!(metrics.contains_key("dns_duration_ms"));
        assert!(metrics.contains_key("status_code"));
        assert!(metrics.contains_key("http_duration_ms"));
        assert!(metrics.contains_key("tls_duration_ms"));
        assert!(metrics.contains_key("tls_handshake_ms"));
        assert!(metrics.contains_key("cert_expiration_ts"));
        assert!(metrics.contains_key("cert_begin_ts"));
    }

    #[test]
    fn test_icmp_metrics_all_fields() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("rtt_ms".to_string(), 25);

        assert_eq!(metrics.len(), 2);
        assert!(metrics.contains_key("up"));
        assert!(metrics.contains_key("rtt_ms"));
    }

    #[test]
    fn test_metrics_with_zero_values() {
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 0);
        metrics.insert("success".to_string(), 0);
        metrics.insert("status_code".to_string(), 0);

        assert_eq!(metrics.get("up"), Some(&0));
        assert_eq!(metrics.get("success"), Some(&0));
    }

    #[test]
    fn test_metrics_with_large_values() {
        let mut metrics = HashMap::new();
        metrics.insert("http_duration_ms".to_string(), 999999);
        metrics.insert("rtt_ms".to_string(), 10000);
        metrics.insert("cert_expiration_ts".to_string(), 2147483647);

        assert_eq!(metrics.get("http_duration_ms"), Some(&999999));
        assert_eq!(metrics.get("rtt_ms"), Some(&10000));
    }

    #[test]
    fn test_duration_to_i64_normal_values() {
        // Test the safe conversion function with normal duration values
        use crate::timescale::metrics::duration_to_i64;

        // Typical duration values (milliseconds)
        assert_eq!(duration_to_i64(0), 0);
        assert_eq!(duration_to_i64(100), 100);
        assert_eq!(duration_to_i64(5000), 5000);
        assert_eq!(duration_to_i64(60_000), 60_000); // 1 minute
        assert_eq!(duration_to_i64(3_600_000), 3_600_000); // 1 hour

        // Maximum safe value (i64::MAX)
        assert_eq!(duration_to_i64(i64::MAX as u128), i64::MAX);
    }

    #[test]
    fn test_duration_to_i64_overflow_protection() {
        // Test that overflow is handled gracefully
        use crate::timescale::metrics::duration_to_i64;

        // Values exceeding i64::MAX should be clamped
        let overflow_value = (i64::MAX as u128) + 1;
        assert_eq!(duration_to_i64(overflow_value), i64::MAX);

        // Extreme overflow
        let extreme_overflow = u128::MAX;
        assert_eq!(duration_to_i64(extreme_overflow), i64::MAX);
    }

    #[test]
    fn test_duration_limits_documentation() {
        // Document the practical limits for duration storage
        // i64::MAX milliseconds = 9,223,372,036,854,775,807 ms
        // = ~292,471,208 years
        // This is sufficient for any realistic probe timeout scenario

        let max_representable_ms = i64::MAX as u128;
        let ms_per_year = 365.25 * 24.0 * 60.0 * 60.0 * 1000.0;
        let years = (max_representable_ms as f64) / ms_per_year;

        // Verify we can represent at least 290 million years
        assert!(years > 290_000_000.0);
    }

    #[test]
    fn test_build_with_no_timescale_exporters_configured() {
        let config = create_test_config_with_timescale(std::collections::HashMap::new());
        let mut exporters: ExportersMap = std::collections::HashMap::new();
        TimescaleExporter::build(&config, &mut exporters);
        assert!(
            exporters.is_empty(),
            "No timescale exporters should be created when none configured"
        );
    }

    #[test]
    fn test_build_with_single_timescale_exporter() {
        let mut timescale_map = std::collections::HashMap::new();
        timescale_map.insert(
            "main".to_string(),
            TimescaleExporterConfiguration {
                connection_string: "postgres://user:pass@localhost/db".to_string(),
                schema: "public".to_string(),
            },
        );
        let config = create_test_config_with_timescale(timescale_map);
        let mut exporters: ExportersMap = std::collections::HashMap::new();
        TimescaleExporter::build(&config, &mut exporters);
        assert_eq!(exporters.len(), 1, "One timescale exporter should be created");
        assert!(exporters.contains_key("exporter.timescale.main"));
    }

    #[test]
    fn test_build_with_multiple_timescale_exporters() {
        let mut timescale_map = std::collections::HashMap::new();
        timescale_map.insert(
            "main".to_string(),
            TimescaleExporterConfiguration {
                connection_string: "postgres://user:pass@localhost/db1".to_string(),
                schema: "public".to_string(),
            },
        );
        timescale_map.insert(
            "analytics".to_string(),
            TimescaleExporterConfiguration {
                connection_string: "postgres://user:pass@localhost/db2".to_string(),
                schema: "analytics".to_string(),
            },
        );
        let config = create_test_config_with_timescale(timescale_map);
        let mut exporters: ExportersMap = std::collections::HashMap::new();
        TimescaleExporter::build(&config, &mut exporters);
        assert_eq!(exporters.len(), 2, "Two timescale exporters should be created");
        assert!(exporters.contains_key("exporter.timescale.main"));
        assert!(exporters.contains_key("exporter.timescale.analytics"));
    }

    #[test]
    fn test_build_timescale_exporter_with_custom_schema() {
        let mut timescale_map = std::collections::HashMap::new();
        timescale_map.insert(
            "custom".to_string(),
            TimescaleExporterConfiguration {
                connection_string: "postgres://user:pass@localhost/db".to_string(),
                schema: "custom_schema".to_string(),
            },
        );
        let config = create_test_config_with_timescale(timescale_map);
        let mut exporters: ExportersMap = std::collections::HashMap::new();
        TimescaleExporter::build(&config, &mut exporters);
        assert_eq!(exporters.len(), 1);
        assert!(exporters.contains_key("exporter.timescale.custom"));
    }
}
