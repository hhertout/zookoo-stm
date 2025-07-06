use std::collections::HashMap;

use opentelemetry::{InstrumentationScope, KeyValue, global};

pub struct MetricsExporter {
    prefix: String,
    labels: HashMap<String, String>,
}

impl MetricsExporter {
    pub fn new(labels: HashMap<String, String>) -> Self {
        MetricsExporter {
            prefix: String::from("probe_"),
            labels,
        }
    }

    pub fn export_icmp_metrics(&self, up: u8, duration: u128) {
        self.set_gauge_metrics(
            String::from(format!("{}target_up", self.prefix)),
            None,
            String::from("target is up or down - 0 is down 1 is up"),
            up as u64,
            &self.labels,
        );

        self.set_gauge_metrics(
            String::from(format!("{}ping_duration", self.prefix)),
            Some(String::from("ms")),
            String::from("dns lookup duration"),
            duration as u64,
            &self.labels,
        );
    }

    pub fn export_metrics(
        &self,
        up: u8,
        success: u8,
        dns_lookup_duration: u128,
        http_status_code: u16,
        http_request_duration: u128,
        http_tls_lookup_duration: Option<u128>,
        http_tls_handshake_duration: Option<u128>,
        tls_cert_expiration_ts: Option<i64>,
        tls_cert_begin_ts: Option<i64>,
    ) {
        self.set_http_request_metrics(&self.labels);

        self.set_gauge_metrics(
            String::from(format!("{}target_up", self.prefix)),
            None,
            String::from("target is up or down - 0 is down 1 is up"),
            up as u64,
            &self.labels,
        );

        self.set_gauge_metrics(
            String::from(format!("{}dns_lookup_duration", self.prefix)),
            Some(String::from("ms")),
            String::from("dns lookup duration"),
            dns_lookup_duration as u64,
            &self.labels,
        );

        self.set_gauge_metrics(
            String::from(format!("{}target_success", self.prefix)),
            None,
            String::from("target match the expected requirements"),
            success as u64,
            &self.labels,
        );

        self.set_gauge_metrics(
            String::from(format!("{}http_status_code", self.prefix)),
            None,
            String::from("http status code"),
            http_status_code as u64,
            &self.labels,
        );

        self.set_gauge_metrics(
            String::from(format!("{}http_request_duration", self.prefix)),
            Some(String::from("ms")),
            String::from("http request total duration"),
            http_request_duration as u64,
            &self.labels,
        );

        if let Some(http_tls_lookup_duration) = http_tls_lookup_duration {
            self.set_gauge_metrics(
                String::from(format!("{}http_tls_lookup_duration", self.prefix)),
                Some(String::from("ms")),
                String::from("tls version"),
                http_tls_lookup_duration as u64,
                &self.labels,
            );
        }

        if let Some(http_tls_handshake_duration) = http_tls_handshake_duration {
            self.set_gauge_metrics(
                String::from(format!("{}http_tls_handshake_duration", self.prefix)),
                Some(String::from("ms")),
                String::from("http tls handshake duration during the request"),
                http_tls_handshake_duration as u64,
                &self.labels,
            );
        }

        if let Some(tls_cert_expiration_ts) = tls_cert_expiration_ts {
            self.set_gauge_metrics(
                String::from(format!("{}cert_expiration", self.prefix)),
                Some(String::from("ts")),
                String::from("certificate expiration timestamp"),
                tls_cert_expiration_ts as u64,
                &self.labels,
            );
        }

        if let Some(tls_cert_begin_ts) = tls_cert_begin_ts {
            self.set_gauge_metrics(
                String::from(format!("{}cert_begin", self.prefix)),
                Some(String::from("ts")),
                String::from("certificate begin timestamp"),
                tls_cert_begin_ts as u64,
                &self.labels,
            );
        }
    }

    pub fn set_up_metrics(&self, value: u8, labels: &HashMap<String, String>) {
        let scope = InstrumentationScope::builder("basic")
            .with_version("1.0")
            .build();

        let meter = global::meter_with_scope(scope);

        let gauge = meter
            .u64_gauge("up")
            .with_description("the target is up or down")
            .build();

        let attr: Vec<KeyValue> = labels
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .collect();

        gauge.record(value as u64, &attr);
    }

    fn set_http_request_metrics(&self, labels: &HashMap<String, String>) {
        let scope = InstrumentationScope::builder("basic")
            .with_version("1.0")
            .build();

        let meter = global::meter_with_scope(scope);

        let counter = meter
            .u64_counter(format!("{}http_request", self.prefix))
            .with_description("total http request")
            .build();

        let attr: Vec<KeyValue> = labels
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .collect();

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
        let scope = InstrumentationScope::builder("basic")
            .with_version("1.0")
            .build();

        let meter = global::meter_with_scope(scope);

        let attr: Vec<KeyValue> = labels
            .iter()
            .map(|(key, value)| KeyValue::new(key.clone(), value.clone()))
            .collect();

        let gauge = if let Some(unit) = unit {
            meter
                .u64_gauge(name)
                .with_description(description)
                .with_unit(unit)
                .build()
        } else {
            meter.u64_gauge(name).with_description(description).build()
        };

        gauge.record(value, &attr);
    }
}
