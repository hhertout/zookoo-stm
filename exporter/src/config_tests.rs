#[cfg(test)]
mod tests {
    use crate::config::*;

    #[test]
    fn test_auth_configuration_with_username_password() {
        let auth = AuthConfiguration {
            username: Some("testuser".to_string()),
            password: Some("testpass".to_string()),
            bearer: None,
        };

        assert_eq!(auth.username, Some("testuser".to_string()));
        assert_eq!(auth.password, Some("testpass".to_string()));
        assert_eq!(auth.bearer, None);
    }

    #[test]
    fn test_auth_configuration_with_bearer() {
        let auth = AuthConfiguration {
            username: None,
            password: None,
            bearer: Some("token123456".to_string()),
        };

        assert_eq!(auth.bearer, Some("token123456".to_string()));
        assert_eq!(auth.username, None);
        assert_eq!(auth.password, None);
    }

    #[test]
    fn test_auth_configuration_empty() {
        let auth = AuthConfiguration {
            username: None,
            password: None,
            bearer: None,
        };

        assert!(auth.username.is_none());
        assert!(auth.password.is_none());
        assert!(auth.bearer.is_none());
    }

    #[test]
    fn test_otel_grpc_config() {
        let config = OtelGrpcExporterConfiguration {
            url: "http://otel-collector:4317".to_string(),
            auth: None,
            tls_insecure: false,
            cert_path: None,
        };

        assert_eq!(config.url, "http://otel-collector:4317");
        assert!(!config.tls_insecure);
        assert!(config.cert_path.is_none());
    }

    #[test]
    fn test_otel_config_with_tls() {
        let config = OtelGrpcExporterConfiguration {
            url: "https://otel-collector:4317".to_string(),
            auth: None,
            tls_insecure: false,
            cert_path: Some("/path/to/ca.crt".to_string()),
        };

        assert!(config.cert_path.is_some());
        assert!(!config.tls_insecure);
    }

    #[test]
    fn test_otel_config_insecure() {
        let config = OtelGrpcExporterConfiguration {
            url: "http://otel-collector:4317".to_string(),
            auth: None,
            tls_insecure: true,
            cert_path: None,
        };

        assert!(config.tls_insecure);
    }

    #[test]
    fn test_prometheus_pushgateway_config() {
        let config = PrometheusPushgatewayConfiguration {
            url: "http://pushgateway:9091".to_string(),
            job: "test-job".to_string(),
            instance: Some("instance-1".to_string()),
            auth: None,
        };

        assert_eq!(config.url, "http://pushgateway:9091");
        assert_eq!(config.job, "test-job");
        assert_eq!(config.instance, Some("instance-1".to_string()));
    }

    #[test]
    fn test_kafka_config() {
        let config = KafkaExporterConfiguration {
            broker: "kafka:9092".to_string(),
            topic: "metrics".to_string(),
            auth: None,
            cert_path: None,
        };

        assert_eq!(config.broker, "kafka:9092");
        assert_eq!(config.topic, "metrics");
    }

    #[test]
    fn test_metrics_exporter_config() {
        let config = MetricsExporterConfiguration {
            endpoint: "http://metrics:8080".to_string(),
        };

        assert_eq!(config.endpoint, "http://metrics:8080");
    }

    #[test]
    fn test_exporter_configuration_empty() {
        let config = ExporterConfiguration {
            otel: None,
            metrics: None,
            kafka: None,
            prometheus: None,
        };

        assert!(config.otel.is_none());
        assert!(config.metrics.is_none());
        assert!(config.kafka.is_none());
        assert!(config.prometheus.is_none());
    }

    #[test]
    fn test_auth_both_methods() {
        let auth = AuthConfiguration {
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            bearer: Some("token".to_string()),
        };

        // Should be able to have both, implementation decides which to use
        assert!(auth.username.is_some());
        assert!(auth.bearer.is_some());
    }

    #[test]
    fn test_config_clone() {
        let config = OtelGrpcExporterConfiguration {
            url: "http://test:4317".to_string(),
            auth: None,
            tls_insecure: false,
            cert_path: None,
        };

        let cloned = config.clone();
        assert_eq!(config.url, cloned.url);
    }
}
