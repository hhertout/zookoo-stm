use std::collections::HashMap;

use crate::http::dns::DnsMetrics;
use crate::http::request::HttpMetrics;
use crate::http::tls::TlsMetrics;

pub struct HttpRequestMetrics {
    pub dns: DnsMetrics,
    pub http: HttpMetrics,
    pub tls: Option<TlsMetrics>,
    pub labels: Option<HashMap<String, String>>,
}
