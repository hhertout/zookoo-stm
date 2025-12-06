use std::collections::HashMap;

use opentelemetry::{InstrumentationScope, KeyValue, global};

pub struct MetricsExporter {
    prefix: String,
    default_labels: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct HttpMetricsParams {
    pub up: u8,
    pub success: u8,
    pub dns_lookup_duration: u128,
    pub tcp_connect_duration: u128,
    pub time_to_first_byte: u128,
    pub content_transfer_duration: u128,
    pub http_status_code: u16,
    pub http_request_duration: u128,
    pub http_tls_lookup_duration: Option<u128>,
    pub http_tls_handshake_duration: Option<u128>,
    pub tls_cert_expiration_ts: Option<i64>,
    pub tls_cert_begin_ts: Option<i64>,
    pub target_labels: HashMap<String, String>,
}

impl MetricsExporter {
    pub fn new(labels: HashMap<String, String>) -> Self {
        MetricsExporter { prefix: String::from("zookoo_"), default_labels: labels }
    }

    /// Merge default labels with target-specific labels (target labels take precedence)
    fn merge_labels(&self, target_labels: &HashMap<String, String>) -> HashMap<String, String> {
        let mut merged = self.default_labels.clone();
        for (key, value) in target_labels {
            merged.insert(key.clone(), value.clone());
        }
        merged
    }

    pub fn export_icmp_metrics(
        &self,
        up: u8,
        duration: u128,
        target_labels: &HashMap<String, String>,
    ) {
        let labels = self.merge_labels(target_labels);

        self.set_gauge_metrics(
            format!("{}target_up", self.prefix),
            None,
            String::from("target is up or down - 0 is down 1 is up"),
            up as u64,
            &labels,
        );

        self.set_gauge_metrics(
            format!("{}ping_duration", self.prefix),
            Some(String::from("ms")),
            String::from("ping duration"),
            duration as u64,
            &labels,
        );

        self.record_histogram(
            format!("{}ping_duration", self.prefix),
            Some(String::from("ms")),
            String::from("ping duration repartition"),
            duration as f64,
            &labels,
        );
    }

    pub fn export_http_metrics(&self, params: HttpMetricsParams) {
        let labels = self.merge_labels(&params.target_labels);

        self.set_http_request_metrics(&labels);

        self.set_gauge_metrics(
            format!("{}target_up", self.prefix),
            None,
            String::from("target is up or down - 0 is down 1 is up"),
            params.up as u64,
            &labels,
        );

        self.set_gauge_metrics(
            format!("{}dns_lookup_duration", self.prefix),
            Some(String::from("ms")),
            String::from("dns lookup duration"),
            params.dns_lookup_duration as u64,
            &labels,
        );

        self.record_histogram(
            format!("{}dns_lookup_duration", self.prefix),
            Some(String::from("ms")),
            String::from("dns lookup duration repartition"),
            params.dns_lookup_duration as f64,
            &labels,
        );

        // TCP connect duration
        self.set_gauge_metrics(
            format!("{}tcp_connect_duration", self.prefix),
            Some(String::from("ms")),
            String::from("tcp connect duration"),
            params.tcp_connect_duration as u64,
            &labels,
        );

        self.record_histogram(
            format!("{}tcp_connect_duration", self.prefix),
            Some(String::from("ms")),
            String::from("tcp connect duration repartition"),
            params.tcp_connect_duration as f64,
            &labels,
        );

        // Time to first byte (TTFB)
        self.set_gauge_metrics(
            format!("{}time_to_first_byte", self.prefix),
            Some(String::from("ms")),
            String::from("time to first byte"),
            params.time_to_first_byte as u64,
            &labels,
        );

        self.record_histogram(
            format!("{}time_to_first_byte", self.prefix),
            Some(String::from("ms")),
            String::from("time to first byte repartition"),
            params.time_to_first_byte as f64,
            &labels,
        );

        // Content transfer duration
        self.set_gauge_metrics(
            format!("{}content_transfer_duration", self.prefix),
            Some(String::from("ms")),
            String::from("content transfer duration"),
            params.content_transfer_duration as u64,
            &labels,
        );

        self.record_histogram(
            format!("{}content_transfer_duration", self.prefix),
            Some(String::from("ms")),
            String::from("content transfer duration repartition"),
            params.content_transfer_duration as f64,
            &labels,
        );

        self.set_gauge_metrics(
            format!("{}target_success", self.prefix),
            None,
            String::from("target match the expected requirements"),
            params.success as u64,
            &labels,
        );

        self.set_gauge_metrics(
            format!("{}http_status_code", self.prefix),
            None,
            String::from("http status code"),
            params.http_status_code as u64,
            &labels,
        );

        self.set_gauge_metrics(
            format!("{}http_request_duration", self.prefix),
            Some(String::from("ms")),
            String::from("http request total duration"),
            params.http_request_duration as u64,
            &labels,
        );

        self.record_histogram(
            format!("{}http_request_duration", self.prefix),
            Some(String::from("ms")),
            String::from("http request total duration repartition"),
            params.http_request_duration as f64,
            &labels,
        );

        if let Some(http_tls_lookup_duration) = params.http_tls_lookup_duration {
            self.set_gauge_metrics(
                format!("{}http_tls_lookup_duration", self.prefix),
                Some(String::from("ms")),
                String::from("tls lookup duration"),
                http_tls_lookup_duration as u64,
                &labels,
            );

            self.record_histogram(
                format!("{}http_tls_lookup_duration", self.prefix),
                Some(String::from("ms")),
                String::from("tls lookup duration repartition"),
                http_tls_lookup_duration as f64,
                &labels,
            );
        }

        if let Some(http_tls_handshake_duration) = params.http_tls_handshake_duration {
            self.set_gauge_metrics(
                format!("{}http_tls_handshake_duration", self.prefix),
                Some(String::from("ms")),
                String::from("http tls handshake duration during the request"),
                http_tls_handshake_duration as u64,
                &labels,
            );

            self.record_histogram(
                format!("{}http_tls_handshake_duration", self.prefix),
                Some(String::from("ms")),
                String::from("http tls handshake duration during the request repartition"),
                http_tls_handshake_duration as f64,
                &labels,
            );
        }

        if let Some(tls_cert_expiration_ts) = params.tls_cert_expiration_ts {
            self.set_gauge_metrics(
                format!("{}cert_expiration", self.prefix),
                Some(String::from("ts")),
                String::from("certificate expiration timestamp"),
                tls_cert_expiration_ts as u64,
                &labels,
            );
        }

        if let Some(tls_cert_begin_ts) = params.tls_cert_begin_ts {
            self.set_gauge_metrics(
                format!("{}cert_begin", self.prefix),
                Some(String::from("ts")),
                String::from("certificate begin timestamp"),
                tls_cert_begin_ts as u64,
                &labels,
            );
        }
    }

    pub fn set_up_metrics(&self, value: u8, labels: &HashMap<String, String>) {
        let scope = InstrumentationScope::builder("basic").with_version("1.0").build();

        let meter = global::meter_with_scope(scope);

        let gauge = meter.u64_gauge("up").with_description("the target is up or down").build();

        let attr: Vec<KeyValue> =
            labels.iter().map(|(key, value)| KeyValue::new(key.clone(), value.clone())).collect();

        gauge.record(value as u64, &attr);
    }

    fn set_http_request_metrics(&self, labels: &HashMap<String, String>) {
        let scope = InstrumentationScope::builder("basic").with_version("1.0").build();

        let meter = global::meter_with_scope(scope);

        let counter = meter
            .u64_counter(format!("{}http_request", self.prefix))
            .with_description("total http request")
            .build();

        let attr: Vec<KeyValue> =
            labels.iter().map(|(key, value)| KeyValue::new(key.clone(), value.clone())).collect();

        counter.add(1, &attr)
    }

    pub fn set_gauge_metrics(
        &self,
        name: String,
        unit: Option<String>,
        description: String,
        value: u64,
        labels: &HashMap<String, String>,
    ) {
        let scope = InstrumentationScope::builder("basic").with_version("1.0").build();

        let meter = global::meter_with_scope(scope);

        let attr: Vec<KeyValue> =
            labels.iter().map(|(key, value)| KeyValue::new(key.clone(), value.clone())).collect();

        let gauge = if let Some(unit) = unit {
            meter.u64_gauge(name).with_description(description).with_unit(unit).build()
        } else {
            meter.u64_gauge(name).with_description(description).build()
        };

        gauge.record(value, &attr);
    }

    pub fn record_histogram(
        &self,
        name: String,
        unit: Option<String>,
        description: String,
        value: f64,
        labels: &HashMap<String, String>,
    ) {
        let scope = InstrumentationScope::builder("basic").with_version("1.0").build();

        let meter = global::meter_with_scope(scope);

        let attr: Vec<KeyValue> =
            labels.iter().map(|(key, value)| KeyValue::new(key.clone(), value.clone())).collect();

        let histogram = if let Some(unit) = unit {
            meter.f64_histogram(name.clone()).with_description(description).with_unit(unit).build()
        } else {
            meter.f64_histogram(name.clone()).with_description(description).build()
        };

        histogram.record(value, &attr);
    }
}
