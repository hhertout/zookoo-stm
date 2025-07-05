use std::time::{Duration, Instant};

use opentelemetry::{
    Context, KeyValue,
    global::ObjectSafeSpan,
    trace::{Status, TraceContextExt},
};
use reqwest::{Client, Version};
use serde::Serialize;

use crate::{config::target::HttpTarget, get_tracer, tracing_new_span_with_context};

#[derive(Debug, Clone, Serialize)]
pub struct HttpMetrics {
    pub up: u8,
    pub success: u8,
    pub duration: Duration,
    pub status_code: u16,
    pub http_version: Option<f32>,
}

impl HttpMetrics {
    pub fn to_logfmt(&self) -> String {
        format!(
            "up={} duration={:?} status_code={} http_version={:?}",
            self.up,
            self.duration.as_millis(),
            self.status_code,
            self.http_version
        )
    }
}

pub async fn http_request(target: &HttpTarget, cx: Context) -> Result<HttpMetrics, reqwest::Error> {
    let mut span =
        tracing_new_span_with_context(get_tracer(), String::from("http_request"), cx.clone());
    span.set_attribute(KeyValue::new("url", target.url.to_string()));
    let cx_with_span = cx.with_span(span);
    let span_ref = cx_with_span.span();

    let url = &target.url;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(15))
        .tcp_keepalive_retries(0)
        .build()
        .unwrap();

    let start = Instant::now();

    let response = match target.method.to_uppercase().as_str() {
        "GET" => client.get(url).send().await?,
        "POST" => client.post(url).send().await?,
        "PUT" => client.put(url).send().await?,
        "PATCH" => client.patch(url).send().await?,
        "DELETE" => client.delete(url).send().await?,
        _ => {
            log::error!("unsupported http method, using get instead");
            client.get(url).send().await?
        }
    };

    let duration = start.elapsed();

    let status = response.status();
    let version = response.version();

    span_ref.set_status(Status::Ok);

    Ok(HttpMetrics {
        up: 1,
        success: (status_code_match(status.as_u16(), target.expected_status_code)) as u8,
        duration: duration,
        status_code: status.as_u16(),
        http_version: version_to_float(version),
    })
}

fn version_to_float(version: Version) -> Option<f32> {
    match version {
        Version::HTTP_09 => Some(0.9),
        Version::HTTP_10 => Some(1.0),
        Version::HTTP_11 => Some(1.1),
        Version::HTTP_2 => Some(2.0),
        Version::HTTP_3 => Some(3.0),
        _ => None,
    }
}

fn status_code_match(status_code: u16, expected_status_code: u16) -> bool {
    status_code == expected_status_code
}
