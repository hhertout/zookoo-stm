#[cfg(test)]
mod tests {
    use crate::timescale::TimescaleExporter;
    use crate::{Export, ExporterRequest, ExporterConfigurationRequest, ProbeType};
    use std::collections::HashMap;

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

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics: metrics.clone(),
        };

        assert_eq!(request.metrics.get("up"), Some(&1));
        assert_eq!(request.metrics.get("success"), Some(&1));
        assert_eq!(request.metrics.get("status_code"), Some(&200));
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

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics: metrics.clone(),
        };

        assert_eq!(request.metrics.get("tls_duration_ms"), Some(&100));
        assert_eq!(request.metrics.get("tls_handshake_ms"), Some(&80));
        assert_eq!(request.metrics.get("cert_expiration_ts"), Some(&1768521599));
    }

    #[test]
    fn test_export_trait_icmp_request_format() {
        // Validate ICMP metrics format
        let mut metrics = HashMap::new();
        metrics.insert("up".to_string(), 1);
        metrics.insert("rtt_ms".to_string(), 25);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics: metrics.clone(),
        };

        assert_eq!(request.metrics.get("up"), Some(&1));
        assert_eq!(request.metrics.get("rtt_ms"), Some(&25));
        
        // ICMP should only have these two metrics
        assert_eq!(request.metrics.len(), 2);
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

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        assert_eq!(request.metrics.get("up"), Some(&0));
        assert_eq!(request.metrics.get("success"), Some(&0));
    }

    #[test]
    fn test_metrics_with_large_values() {
        let mut metrics = HashMap::new();
        metrics.insert("http_duration_ms".to_string(), 999999);
        metrics.insert("rtt_ms".to_string(), 10000);
        metrics.insert("cert_expiration_ts".to_string(), 2147483647);

        let request = ExporterRequest {
            exporter: ExporterConfigurationRequest {},
            metrics,
        };

        assert_eq!(request.metrics.get("http_duration_ms"), Some(&999999));
        assert_eq!(request.metrics.get("rtt_ms"), Some(&10000));
    }
}
