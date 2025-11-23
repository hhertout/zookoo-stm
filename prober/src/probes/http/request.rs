use crate::{observability::child_span_from_context, config::target::HttpTarget};
use opentelemetry::{
    Context, KeyValue,
    trace::{Status, TraceContextExt},
};
use reqwest::{Client, Version};
use serde::Serialize;
use std::time::{Duration, Instant};

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
            "up={} duration={:?} status_code={} http_version={:?} success={}",
            self.up,
            self.duration.as_millis(),
            self.status_code,
            self.http_version,
            self.success,
        )
    }
}

pub async fn http_request(client: &Client, target: &HttpTarget, cx: Context) -> Result<HttpMetrics, reqwest::Error> {
    let span_attr = vec![KeyValue::new("url", target.url.to_string())];
    let cx_with_span = child_span_from_context("http_request", cx.clone(), span_attr);
    let span_ref = cx_with_span.span();

    let url = &target.url;

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
    span_ref.set_status(Status::Ok);

    let status_code = response.status();
    let version = response.version();
    let status = status_code_match(status_code.as_u16(), target.expected_status_code) as u8;

    Ok(HttpMetrics {
        up: 1,
        success: status,
        duration: duration,
        status_code: status_code.as_u16(),
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
