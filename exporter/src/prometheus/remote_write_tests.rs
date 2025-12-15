#[cfg(test)]
mod tests {
    use super::super::remote_write::*;
    use configuration::model::exporter::AuthConfiguration;
    use std::collections::HashMap;

    #[test]
    fn test_create_remote_write_exporter() {
        let config = PrometheusRemoteWriteConfig {
            url: "http://localhost:9999/api/v1/metrics/write".to_string(),
            job: "test-job".to_string(),
            instance: Some("test-instance".to_string()),
            auth: None,
            extra_labels: HashMap::new(),
        };

        let exporter = PrometheusRemoteWrite::new(config);
        assert!(exporter.is_ok());
    }

    #[test]
    fn test_create_sample() {
        let sample = Sample { value: 42.0, timestamp: 1234567890 };

        assert_eq!(sample.value, 42.0);
        assert_eq!(sample.timestamp, 1234567890);
    }

    #[test]
    fn test_create_label() {
        let label = Label { name: "test_label".to_string(), value: "test_value".to_string() };

        assert_eq!(label.name, "test_label");
        assert_eq!(label.value, "test_value");
    }

    #[test]
    fn test_create_time_series() {
        let labels = vec![
            Label { name: "__name__".to_string(), value: "test_metric".to_string() },
            Label { name: "job".to_string(), value: "test-job".to_string() },
        ];

        let samples = vec![Sample { value: 123.45, timestamp: 1234567890 }];

        let time_series = TimeSeries { labels, samples };

        assert_eq!(time_series.labels.len(), 2);
        assert_eq!(time_series.samples.len(), 1);
    }

    #[test]
    fn test_create_write_request() {
        let time_series = TimeSeries {
            labels: vec![Label { name: "__name__".to_string(), value: "test_metric".to_string() }],
            samples: vec![Sample { value: 100.0, timestamp: 1234567890 }],
        };

        let write_request = WriteRequest { timeseries: vec![time_series] };

        assert_eq!(write_request.timeseries.len(), 1);
    }

    #[test]
    fn test_config_with_auth() {
        let mut auth = AuthConfiguration {
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            bearer: None,
        };

        assert_eq!(auth.username, Some("user".to_string()));
        assert_eq!(auth.password, Some("pass".to_string()));
        assert_eq!(auth.bearer, None);

        auth.bearer = Some("token123".to_string());
        assert_eq!(auth.bearer, Some("token123".to_string()));
    }

    #[test]
    fn test_config_with_extra_labels() {
        let mut extra_labels = HashMap::new();
        extra_labels.insert("environment".to_string(), "production".to_string());
        extra_labels.insert("region".to_string(), "us-east-1".to_string());

        let config = PrometheusRemoteWriteConfig {
            url: "http://localhost:9999/api/v1/metrics/write".to_string(),
            job: "test-job".to_string(),
            instance: Some("test-instance".to_string()),
            auth: None,
            extra_labels: extra_labels.clone(),
        };

        assert_eq!(config.extra_labels.len(), 2);
        assert_eq!(config.extra_labels.get("environment"), Some(&"production".to_string()));
        assert_eq!(config.extra_labels.get("region"), Some(&"us-east-1".to_string()));
    }

    #[test]
    fn test_protobuf_encoding() {
        use prost::Message;

        let write_request = WriteRequest {
            timeseries: vec![TimeSeries {
                labels: vec![
                    Label {
                        name: "__name__".to_string(),
                        value: "http_request_duration_seconds".to_string(),
                    },
                    Label { name: "job".to_string(), value: "zookoo-stm".to_string() },
                    Label { name: "status".to_string(), value: "200".to_string() },
                ],
                samples: vec![Sample { value: 0.234, timestamp: 1700000000000 }],
            }],
        };

        let mut buf = Vec::new();
        let result = write_request.encode(&mut buf);
        assert!(result.is_ok());
        assert!(!buf.is_empty());

        // Decode back to verify
        let decoded = WriteRequest::decode(&buf[..]);
        assert!(decoded.is_ok());
        let decoded_req = decoded.unwrap();
        assert_eq!(decoded_req.timeseries.len(), 1);
        assert_eq!(decoded_req.timeseries[0].labels.len(), 3);
    }

    #[test]
    fn test_snappy_compression() {
        use prost::Message;

        let write_request = WriteRequest {
            timeseries: vec![TimeSeries {
                labels: vec![Label {
                    name: "__name__".to_string(),
                    value: "test_metric".to_string(),
                }],
                samples: vec![Sample { value: 42.0, timestamp: 1234567890 }],
            }],
        };

        let mut buf = Vec::new();
        write_request.encode(&mut buf).unwrap();

        // Compress with Snappy
        let compressed = snap::raw::Encoder::new().compress_vec(&buf);
        assert!(compressed.is_ok());
        let compressed_data = compressed.unwrap();

        // Decompress to verify
        let decompressed = snap::raw::Decoder::new().decompress_vec(&compressed_data);
        assert!(decompressed.is_ok());
        assert_eq!(decompressed.unwrap(), buf);
    }

    #[test]
    fn test_multiple_metrics_in_write_request() {
        let timeseries = vec![
            TimeSeries {
                labels: vec![
                    Label {
                        name: "__name__".to_string(),
                        value: "http_requests_total".to_string(),
                    },
                    Label { name: "job".to_string(), value: "zookoo".to_string() },
                ],
                samples: vec![Sample { value: 1234.0, timestamp: 1700000000000 }],
            },
            TimeSeries {
                labels: vec![
                    Label {
                        name: "__name__".to_string(),
                        value: "http_request_duration_seconds".to_string(),
                    },
                    Label { name: "job".to_string(), value: "zookoo".to_string() },
                ],
                samples: vec![Sample { value: 0.456, timestamp: 1700000000000 }],
            },
        ];

        let write_request = WriteRequest { timeseries };

        assert_eq!(write_request.timeseries.len(), 2);
    }
}
