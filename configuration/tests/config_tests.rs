use configuration::model::*;

#[cfg(test)]
mod configuration_tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal_configuration() {
        let toml = r#"
[defaults]
log_level = "info"

[defaults.self_monitoring]
enable = false

[exporter]
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize minimal config");
        
        assert_eq!(config.defaults.log_level, "info");
        assert_eq!(config.defaults.self_monitoring.enable, false);
        assert!(config.http.is_none());
        assert!(config.icmp.is_none());
        assert!(config.discovery.is_none());
    }

    #[test]
    fn test_deserialize_full_configuration() {
        let toml = r#"
[defaults]
log_level = "debug"
probe_zone = "eu-west-1"

[defaults.probe_location]
latitude = 48.8566
longitude = 2.3522

[defaults.self_monitoring]
enable = true
otel_endpoint = "https://otel.example.com:4317"
pyroscope_endpoint = "https://pyroscope.example.com:4040"
service_name = "zookoo-prod"
env = "production"
tls_ignore = false

[exporter]

[exporter.otel]
url = "https://otel-collector.example.com:4317"
tls_insecure = false

[exporter.otel.auth]
username = "admin"
password = "secret"

[exporter.metrics]
endpoint = "http://prometheus.example.com:9090"

[exporter.kafka]
broker = "kafka.example.com:9092"
topic = "metrics"

[[http.targets]]
method = "GET"
url = "https://api.example.com/health"
expected_status_code = 200
timeout_sec = 10
scrape_interval = "30s"
follow_redirect = true
skip_tls = false

[[http.targets]]
method = "POST"
url = "https://api2.example.com/check"
expected_status_code = 201
timeout_sec = 5
scrape_interval = "1m"

[http.targets.headers]
"Authorization" = "Bearer token123"
"X-Custom-Header" = "value"

[http.targets.labels]
"environment" = "production"
"team" = "platform"

[http.targets.auth]
bearer = "token456"

[[icmp.targets]]
ipv4 = "8.8.8.8"
timeout_sec = 5
scrape_interval = "10s"

[icmp.targets.labels]
"name" = "google-dns"

[[icmp.targets]]
fqdn = "cloudflare.com"
timeout_sec = 10
scrape_interval = "30s"

[discovery]

[discovery.file]

[[discovery.file.http]]
path = ["/etc/zookoo/discovery/http1.json", "/etc/zookoo/discovery/http2.json"]
scrape_interval = "1m"

[discovery.file.http.labels]
"source" = "discovery"

[[discovery.file.icmp]]
path = ["/etc/zookoo/discovery/icmp.json"]
scrape_interval = "5m"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize full config");
        
        // Test defaults
        assert_eq!(config.defaults.log_level, "debug");
        assert_eq!(config.defaults.probe_zone, Some("eu-west-1".to_string()));
        
        let location = config.defaults.probe_location.as_ref().unwrap();
        assert_eq!(location.latitude, 48.8566);
        assert_eq!(location.longitude, 2.3522);
        
        let self_mon = &config.defaults.self_monitoring;
        assert_eq!(self_mon.enable, true);
        assert_eq!(self_mon.otel_endpoint, "https://otel.example.com:4317");
        assert_eq!(self_mon.pyroscope_endpoint, "https://pyroscope.example.com:4040");
        assert_eq!(self_mon.service_name, "zookoo-prod");
        assert_eq!(self_mon.env, "production");
        assert_eq!(self_mon.tls_ignore, false);
        
        // Test exporter
        let otel = config.exporter.otel.as_ref().unwrap();
        assert_eq!(otel.url, "https://otel-collector.example.com:4317");
        assert_eq!(otel.tls_insecure, false);
        
        let auth = otel.auth.as_ref().unwrap();
        assert_eq!(auth.username, Some("admin".to_string()));
        assert_eq!(auth.password, Some("secret".to_string()));
        
        let metrics = config.exporter.metrics.as_ref().unwrap();
        assert_eq!(metrics.endpoint, "http://prometheus.example.com:9090");
        
        let kafka = config.exporter.kafka.as_ref().unwrap();
        assert_eq!(kafka.broker, "kafka.example.com:9092");
        assert_eq!(kafka.topic, "metrics");
        
        // Test HTTP targets
        let http_config = config.http.as_ref().unwrap();
        assert_eq!(http_config.targets.len(), 2);
        
        let http_target1 = &http_config.targets[0];
        assert_eq!(http_target1.method, "GET");
        assert_eq!(http_target1.url, "https://api.example.com/health");
        assert_eq!(http_target1.expected_status_code, 200);
        assert_eq!(http_target1.timeout_sec, 10);
        assert_eq!(http_target1.follow_redirect, true);
        assert_eq!(http_target1.skip_tls, false);
        
        let http_target2 = &http_config.targets[1];
        assert_eq!(http_target2.method, "POST");
        assert_eq!(http_target2.url, "https://api2.example.com/check");
        assert_eq!(http_target2.expected_status_code, 201);
        
        let headers = http_target2.headers.as_ref().unwrap();
        assert_eq!(headers.get("Authorization"), Some(&"Bearer token123".to_string()));
        
        let labels = http_target2.labels.as_ref().unwrap();
        assert_eq!(labels.get("environment"), Some(&"production".to_string()));
        assert_eq!(labels.get("team"), Some(&"platform".to_string()));
        
        let target2_auth = http_target2.auth.as_ref().unwrap();
        assert_eq!(target2_auth.bearer, Some("token456".to_string()));
        
        // Test ICMP targets
        let icmp_config = config.icmp.as_ref().unwrap();
        assert_eq!(icmp_config.targets.len(), 2);
        
        let icmp_target1 = &icmp_config.targets[0];
        assert_eq!(icmp_target1.ipv4, Some("8.8.8.8".to_string()));
        assert_eq!(icmp_target1.timeout_sec, 5);
        
        let icmp_labels = icmp_target1.labels.as_ref().unwrap();
        assert_eq!(icmp_labels.get("name"), Some(&"google-dns".to_string()));
        
        let icmp_target2 = &icmp_config.targets[1];
        assert_eq!(icmp_target2.fqdn, Some("cloudflare.com".to_string()));
        assert_eq!(icmp_target2.timeout_sec, 10);
        
        // Test discovery
        let discovery = config.discovery.as_ref().unwrap();
        let discovery_file = discovery.file.as_ref().unwrap();
        
        let http_discovery = discovery_file.http.as_ref().unwrap();
        assert_eq!(http_discovery.len(), 1);
        assert_eq!(http_discovery[0].path.len(), 2);
        assert_eq!(http_discovery[0].path[0], "/etc/zookoo/discovery/http1.json");
        
        let icmp_discovery = discovery_file.icmp.as_ref().unwrap();
        assert_eq!(icmp_discovery.len(), 1);
        assert_eq!(icmp_discovery[0].path[0], "/etc/zookoo/discovery/icmp.json");
    }

    #[test]
    fn test_deserialize_defaults_with_all_default_values() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        // Test default values
        assert_eq!(config.defaults.log_level, "info");
        assert_eq!(config.defaults.self_monitoring.enable, false);
        assert_eq!(config.defaults.self_monitoring.otel_endpoint, "http://localhost:4317");
        assert_eq!(config.defaults.self_monitoring.pyroscope_endpoint, "http://localhost:9999");
        assert_eq!(config.defaults.self_monitoring.service_name, "zookoo");
        assert_eq!(config.defaults.self_monitoring.env, "development");
        assert_eq!(config.defaults.self_monitoring.tls_ignore, false);
    }

    #[test]
    fn test_deserialize_http_target_defaults() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
url = "https://example.com"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let http_config = config.http.as_ref().unwrap();
        let target = &http_config.targets[0];
        
        // Test default values for HTTP target
        assert_eq!(target.method, "GET");
        assert_eq!(target.expected_status_code, 200);
        assert_eq!(target.timeout_sec, 15);
        assert_eq!(target.follow_redirect, false);
        assert_eq!(target.skip_tls, false);
    }

    #[test]
    fn test_deserialize_icmp_target_defaults() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[icmp.targets]]
ipv4 = "1.1.1.1"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let icmp_config = config.icmp.as_ref().unwrap();
        let target = &icmp_config.targets[0];
        
        // Test default values for ICMP target
        assert_eq!(target.timeout_sec, 15);
    }

    #[test]
    fn test_deserialize_scrape_intervals() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
url = "https://test1.com"
scrape_interval = "5s"

[[http.targets]]
url = "https://test2.com"
scrape_interval = "10s"

[[http.targets]]
url = "https://test3.com"
scrape_interval = "30s"

[[http.targets]]
url = "https://test4.com"
scrape_interval = "1m"

[[http.targets]]
url = "https://test5.com"
scrape_interval = "5m"

[[http.targets]]
url = "https://test6.com"
scrape_interval = "10m"

[[http.targets]]
url = "https://test7.com"
scrape_interval = "30m"

[[http.targets]]
url = "https://test8.com"
scrape_interval = "1h"

[[http.targets]]
url = "https://test9.com"
scrape_interval = "12h"

[[http.targets]]
url = "https://test10.com"
scrape_interval = "1d"

[[http.targets]]
url = "https://test11.com"
scrape_interval = "7d"

[[http.targets]]
url = "https://test12.com"
scrape_interval = "30d"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let http_config = config.http.as_ref().unwrap();
        assert_eq!(http_config.targets.len(), 12);
        
        // Verify all scrape intervals can be deserialized
        use std::time::Duration;
        
        assert_eq!(http_config.targets[0].scrape_interval.to_duration(), Duration::from_secs(5));
        assert_eq!(http_config.targets[1].scrape_interval.to_duration(), Duration::from_secs(10));
        assert_eq!(http_config.targets[2].scrape_interval.to_duration(), Duration::from_secs(30));
        assert_eq!(http_config.targets[3].scrape_interval.to_duration(), Duration::from_secs(60));
        assert_eq!(http_config.targets[4].scrape_interval.to_duration(), Duration::from_secs(300));
        assert_eq!(http_config.targets[5].scrape_interval.to_duration(), Duration::from_secs(600));
        assert_eq!(http_config.targets[6].scrape_interval.to_duration(), Duration::from_secs(1800));
        assert_eq!(http_config.targets[7].scrape_interval.to_duration(), Duration::from_secs(3600));
        assert_eq!(http_config.targets[8].scrape_interval.to_duration(), Duration::from_secs(43200));
        assert_eq!(http_config.targets[9].scrape_interval.to_duration(), Duration::from_secs(86400));
        assert_eq!(http_config.targets[10].scrape_interval.to_duration(), Duration::from_secs(604800));
        assert_eq!(http_config.targets[11].scrape_interval.to_duration(), Duration::from_secs(2592000));
    }

    #[test]
    fn test_deserialize_complex_labels() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
url = "https://example.com"

[http.targets.labels]
"env" = "production"
"region" = "us-east-1"
"team" = "platform"
"service" = "api"
"version" = "v1.2.3"
"custom-label-with-dash" = "value-with-dash"
"custom.label.with.dot" = "value.with.dot"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let http_config = config.http.as_ref().unwrap();
        let labels = http_config.targets[0].labels.as_ref().unwrap();
        
        assert_eq!(labels.len(), 7);
        assert_eq!(labels.get("env"), Some(&"production".to_string()));
        assert_eq!(labels.get("region"), Some(&"us-east-1".to_string()));
        assert_eq!(labels.get("team"), Some(&"platform".to_string()));
        assert_eq!(labels.get("service"), Some(&"api".to_string()));
        assert_eq!(labels.get("version"), Some(&"v1.2.3".to_string()));
        assert_eq!(labels.get("custom-label-with-dash"), Some(&"value-with-dash".to_string()));
        assert_eq!(labels.get("custom.label.with.dot"), Some(&"value.with.dot".to_string()));
    }

    #[test]
    fn test_deserialize_multiple_auth_types() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
url = "https://example1.com"

[http.targets.auth]
username = "user1"
password = "pass1"

[[http.targets]]
url = "https://example2.com"

[http.targets.auth]
bearer = "token123"

[[http.targets]]
url = "https://example3.com"

[http.targets.auth]
username = "user2"
bearer = "token456"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let http_config = config.http.as_ref().unwrap();
        
        let auth1 = http_config.targets[0].auth.as_ref().unwrap();
        assert_eq!(auth1.username, Some("user1".to_string()));
        assert_eq!(auth1.password, Some("pass1".to_string()));
        assert_eq!(auth1.bearer, None);
        
        let auth2 = http_config.targets[1].auth.as_ref().unwrap();
        assert_eq!(auth2.username, None);
        assert_eq!(auth2.password, None);
        assert_eq!(auth2.bearer, Some("token123".to_string()));
        
        let auth3 = http_config.targets[2].auth.as_ref().unwrap();
        assert_eq!(auth3.username, Some("user2".to_string()));
        assert_eq!(auth3.bearer, Some("token456".to_string()));
    }

    #[test]
    fn test_deserialize_exporter_with_tls_insecure() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[exporter.otel]
url = "https://otel.example.com:4317"
tls_insecure = true
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let otel = config.exporter.otel.as_ref().unwrap();
        assert_eq!(otel.tls_insecure, true);
    }

    #[test]
    fn test_deserialize_exporter_with_cert_path() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[exporter.otel]
url = "https://otel.example.com:4317"
cert_path = "/etc/ssl/certs/ca-bundle.crt"

[exporter.kafka]
broker = "kafka.example.com:9092"
topic = "metrics"
cert_path = "/etc/ssl/certs/kafka-ca.crt"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let otel = config.exporter.otel.as_ref().unwrap();
        assert_eq!(otel.cert_path, Some("/etc/ssl/certs/ca-bundle.crt".to_string()));
        
        let kafka = config.exporter.kafka.as_ref().unwrap();
        assert_eq!(kafka.cert_path, Some("/etc/ssl/certs/kafka-ca.crt".to_string()));
    }

    #[test]
    fn test_deserialize_mixed_http_methods() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
method = "GET"
url = "https://example.com/get"

[[http.targets]]
method = "POST"
url = "https://example.com/post"

[[http.targets]]
method = "PUT"
url = "https://example.com/put"

[[http.targets]]
method = "PATCH"
url = "https://example.com/patch"

[[http.targets]]
method = "DELETE"
url = "https://example.com/delete"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let http_config = config.http.as_ref().unwrap();
        assert_eq!(http_config.targets.len(), 5);
        assert_eq!(http_config.targets[0].method, "GET");
        assert_eq!(http_config.targets[1].method, "POST");
        assert_eq!(http_config.targets[2].method, "PUT");
        assert_eq!(http_config.targets[3].method, "PATCH");
        assert_eq!(http_config.targets[4].method, "DELETE");
    }

    #[test]
    fn test_deserialize_mixed_icmp_targets() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[icmp.targets]]
ipv4 = "8.8.8.8"
scrape_interval = "30s"

[[icmp.targets]]
fqdn = "example.com"
scrape_interval = "1m"

[[icmp.targets]]
ipv4 = "1.1.1.1"
fqdn = "cloudflare.com"
scrape_interval = "5m"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let icmp_config = config.icmp.as_ref().unwrap();
        assert_eq!(icmp_config.targets.len(), 3);
        
        assert_eq!(icmp_config.targets[0].ipv4, Some("8.8.8.8".to_string()));
        assert_eq!(icmp_config.targets[0].fqdn, None);
        
        assert_eq!(icmp_config.targets[1].ipv4, None);
        assert_eq!(icmp_config.targets[1].fqdn, Some("example.com".to_string()));
        
        assert_eq!(icmp_config.targets[2].ipv4, Some("1.1.1.1".to_string()));
        assert_eq!(icmp_config.targets[2].fqdn, Some("cloudflare.com".to_string()));
    }

    #[test]
    fn test_deserialize_empty_optional_fields() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
url = "https://example.com"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let http_config = config.http.as_ref().unwrap();
        let target = &http_config.targets[0];
        
        assert!(target.headers.is_none());
        assert!(target.labels.is_none());
        assert!(target.auth.is_none());
        assert!(config.discovery.is_none());
    }

    #[test]
    fn test_deserialize_discovery_with_multiple_paths() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[discovery]

[discovery.file]

[[discovery.file.http]]
path = [
    "/etc/zookoo/http/targets1.json",
    "/etc/zookoo/http/targets2.json",
    "/etc/zookoo/http/targets3.json"
]
scrape_interval = "1m"

[discovery.file.http.labels]
"source" = "file-discovery"
"type" = "http"

[[discovery.file.icmp]]
path = [
    "/etc/zookoo/icmp/targets1.json",
    "/etc/zookoo/icmp/targets2.json"
]
scrape_interval = "5m"

[discovery.file.icmp.labels]
"source" = "file-discovery"
"type" = "icmp"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let discovery = config.discovery.as_ref().unwrap();
        let file = discovery.file.as_ref().unwrap();
        
        let http = file.http.as_ref().unwrap();
        assert_eq!(http.len(), 1);
        assert_eq!(http[0].path.len(), 3);
        assert_eq!(http[0].path[0], "/etc/zookoo/http/targets1.json");
        assert_eq!(http[0].path[1], "/etc/zookoo/http/targets2.json");
        assert_eq!(http[0].path[2], "/etc/zookoo/http/targets3.json");
        
        let http_labels = http[0].labels.as_ref().unwrap();
        assert_eq!(http_labels.get("source"), Some(&"file-discovery".to_string()));
        assert_eq!(http_labels.get("type"), Some(&"http".to_string()));
        
        let icmp = file.icmp.as_ref().unwrap();
        assert_eq!(icmp.len(), 1);
        assert_eq!(icmp[0].path.len(), 2);
        
        let icmp_labels = icmp[0].labels.as_ref().unwrap();
        assert_eq!(icmp_labels.get("type"), Some(&"icmp".to_string()));
    }

    #[test]
    #[should_panic(expected = "missing field")]
    fn test_deserialize_missing_required_fields() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]
"#;

        // Should panic because 'exporter' is required
        let _config: Configuration = toml::from_str(toml).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_deserialize_invalid_scrape_interval() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
url = "https://example.com"
scrape_interval = "invalid"
"#;

        let _config: Configuration = toml::from_str(toml).unwrap();
    }

    #[test]
    fn test_deserialize_with_special_characters_in_url() {
        let toml = r#"
[defaults]
[defaults.self_monitoring]

[exporter]

[[http.targets]]
url = "https://api.example.com/v1/users?filter=active&limit=100&offset=0"
"#;

        let config: Configuration = toml::from_str(toml).expect("Failed to deserialize");
        
        let http_config = config.http.as_ref().unwrap();
        assert_eq!(
            http_config.targets[0].url,
            "https://api.example.com/v1/users?filter=active&limit=100&offset=0"
        );
    }
}
