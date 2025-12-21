//! Low-level HTTP Client
//!
//! Performs HTTP requests with phase-by-phase timing using hyper.
//! Measures DNS, TCP connect, TLS handshake, TTFB, and content transfer separately.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use base64::Engine;
use bytes::Bytes;
use configuration::DEFAULT_SOURCE;
use http_body_util::{BodyExt, Empty};
use hyper::header::{AUTHORIZATION, HOST};
use hyper::{Method, Request};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tracing::{Instrument, info_span};

use super::metrics::HttpProbeMetrics;
use super::resolver::{DnsResolver, extract_host, extract_port};
use super::tls::TlsHandler;
/// HTTP request configuration
#[derive(Debug, Clone)]
pub struct HttpRequestConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    pub expected_status_code: u16,
    pub timeout_sec: u16,
    pub skip_tls: bool,
    pub follow_redirect: bool,
    pub auth: Option<AuthConfig>,
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub bearer: Option<String>,
}

/// Low-level HTTP client with phase timing
pub struct HttpClient {
    resolver: DnsResolver,
}

impl HttpClient {
    #[tracing::instrument(name = "http_client_new")]
    pub fn new() -> Self {
        Self { resolver: DnsResolver::new() }
    }

    /// Execute an HTTP request with full phase timing
    #[tracing::instrument(
        name = "http_client_execute",
        skip(self, config),
        fields(url = %config.url, method = %config.method)
    )]
    pub async fn execute(&self, config: &HttpRequestConfig) -> HttpProbeMetrics {
        let total_start = Instant::now();
        let mut metrics = HttpProbeMetrics::new();

        // Parse URL components
        let host = match extract_host(&config.url) {
            Ok(h) => h,
            Err(e) => {
                log::error!(
                    "source={} event=invalid_url url={} err={}",
                    DEFAULT_SOURCE,
                    config.url,
                    e
                );
                return metrics;
            }
        };

        let port = match extract_port(&config.url) {
            Ok(p) => p,
            Err(e) => {
                log::error!(
                    "source={} event=invalid_port url={} err={}",
                    DEFAULT_SOURCE,
                    config.url,
                    e
                );
                return metrics;
            }
        };

        let is_https = config.url.starts_with("https://");

        // === Phase 1: DNS Resolution ===
        let dns_span = info_span!("dns_resolution", host = %host);
        log::info!("source={} event=dns_start ip_protocol=ipv4 host={}", DEFAULT_SOURCE, host);
        let (ip_addr, dns_duration) = match self
            .resolver
            .resolve_first_ipv4(&host)
            .instrument(dns_span)
            .await
        {
            Ok((ip, dur)) => (ip, dur),
            Err(e) => {
                log::error!("source={} event=dns_failed host={} err={}", DEFAULT_SOURCE, host, e);
                metrics.dns_duration = total_start.elapsed();
                return metrics;
            }
        };
        metrics.dns_duration = dns_duration;
        metrics.resolved_ip = Some(ip_addr.to_string());
        log::info!("source={} event=dns_complete host={} ip={}", DEFAULT_SOURCE, host, ip_addr);

        // === Phase 2: TCP Connect ===
        let tcp_start = Instant::now();
        let socket_addr = SocketAddr::new(ip_addr, port);

        let tcp_span = info_span!("tcp_connect", ip = %ip_addr, port = port);
        let tcp_stream = match tokio::time::timeout(
            Duration::from_secs(config.timeout_sec as u64),
            TcpStream::connect(socket_addr),
        )
        .instrument(tcp_span)
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                log::error!(
                    "source={} event=tcp_connect_failed addr={} err={}",
                    DEFAULT_SOURCE,
                    socket_addr,
                    e
                );
                metrics.tcp_connect_duration = tcp_start.elapsed();
                return metrics;
            }
            Err(_) => {
                log::error!(
                    "source={} event=tcp_connect_timeout addr={}",
                    DEFAULT_SOURCE,
                    socket_addr
                );
                metrics.tcp_connect_duration = tcp_start.elapsed();
                return metrics;
            }
        };
        metrics.tcp_connect_duration = tcp_start.elapsed();
        metrics.up = true; // TCP connection succeeded

        // === Phase 3: TLS Handshake (if HTTPS) ===
        if is_https {
            let tls_handler = match TlsHandler::new(config.skip_tls) {
                Ok(h) => h,
                Err(e) => {
                    log::error!("source={} event=tls_init_failed err={}", DEFAULT_SOURCE, e);
                    return metrics;
                }
            };

            let tls_span = info_span!("tls_handshake", host = %host, skip_verify = config.skip_tls);
            let tls_result =
                match tls_handler.handshake(tcp_stream, &host).instrument(tls_span).await {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!(
                            "source={} event=tls_handshake_failed host={} err={}",
                            DEFAULT_SOURCE,
                            host,
                            e
                        );
                        return metrics;
                    }
                };

            metrics.tls_handshake_duration = Some(tls_result.handshake_duration);
            metrics.tls_version = Some(tls_result.cert_info.tls_version);
            metrics.cert_expiration_ts = tls_result.cert_info.not_after;
            metrics.cert_begin_ts = tls_result.cert_info.not_before;
            metrics.cert_issuer = tls_result.cert_info.issuer;
            metrics.cert_subject = tls_result.cert_info.subject;

            // Execute HTTP request over TLS
            self.execute_http_request(TokioIo::new(tls_result.stream), config, &host, &mut metrics)
                .await;
        } else {
            // Execute HTTP request over plain TCP
            self.execute_http_request(TokioIo::new(tcp_stream), config, &host, &mut metrics).await;
        }

        metrics.total_duration = total_start.elapsed();
        metrics
    }

    /// Execute the HTTP request and measure TTFB + content transfer
    async fn execute_http_request<S>(
        &self,
        io: TokioIo<S>,
        config: &HttpRequestConfig,
        host: &str,
        metrics: &mut HttpProbeMetrics,
    ) where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let request_span = info_span!("http_request", host = %host, method = %config.method);
        log::info!("source={} event=request_start url={} method={}", DEFAULT_SOURCE, config.url, config.method);
        
        async {
            let ttfb_start = Instant::now();

            // Build HTTP connection
            let (mut sender, conn) = match hyper::client::conn::http1::handshake(io).await {
                Ok(r) => r,
                Err(e) => {
                    log::error!("event=error msg=http_handshake_failed err={}", e);
                    return;
                }
            };

            // Spawn connection driver (keep it correlated to this request)
            tokio::spawn(
                async move {
                    if let Err(e) = conn.await {
                        log::error!("event=error msg=http_connection_error err={}", e);
                    }
                }
                .in_current_span(),
            );

            // Build request
            let method = match config.method.to_uppercase().as_str() {
                "GET" => Method::GET,
                "POST" => Method::POST,
                "PUT" => Method::PUT,
                "DELETE" => Method::DELETE,
                "HEAD" => Method::HEAD,
                "OPTIONS" => Method::OPTIONS,
                "PATCH" => Method::PATCH,
                _ => Method::GET,
            };

            let uri = match config.url.parse::<hyper::Uri>() {
                Ok(u) => u,
                Err(e) => {
                    log::error!("event=error msg=invalid_uri url={} err={}", config.url, e);
                    return;
                }
            };

            let path_and_query = uri.path_and_query().map(|p| p.as_str()).unwrap_or("/");

            let mut req_builder =
                Request::builder().method(method).uri(path_and_query).header(HOST, host);

            // Add custom headers
            if let Some(headers) = &config.headers {
                for (key, value) in headers {
                    req_builder = req_builder.header(key.as_str(), value.as_str());
                }
            }

            // Add authentication
            if let Some(auth) = &config.auth {
                if let Some(bearer) = &auth.bearer {
                    req_builder = req_builder.header(AUTHORIZATION, format!("Bearer {}", bearer));
                } else if let (Some(user), Some(pass)) = (&auth.username, &auth.password) {
                    let credentials = base64::engine::general_purpose::STANDARD
                        .encode(format!("{}:{}", user, pass));
                    req_builder =
                        req_builder.header(AUTHORIZATION, format!("Basic {}", credentials));
                }
            }

            let request = match req_builder.body(Empty::<Bytes>::new()) {
                Ok(r) => r,
                Err(e) => {
                    log::error!("source={} event=request_build_failed err={}", DEFAULT_SOURCE, e);
                    return;
                }
            };

            // Send request and wait for response headers
            let response = match sender.send_request(request).await {
                Ok(r) => r,
                Err(e) => {
                    log::error!("source={} event=request_failed err={}", DEFAULT_SOURCE, e);
                    return;
                }
            };

            
            metrics.time_to_first_byte = ttfb_start.elapsed();
            metrics.status_code = response.status().as_u16();
            metrics.http_version = format!("{:?}", response.version());
            
            log::info!("source={} event=request_complete status_code={}", DEFAULT_SOURCE, response.status().as_u16());
            
            // Get content length if available
            metrics.content_length = response
                .headers()
                .get(hyper::header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok());

            // === Phase 5: Content Transfer ===
            let content_start = Instant::now();

            // Consume the body
            let body = response.into_body();
            let content_span = info_span!("content_transfer");
            if let Err(e) = body.collect().instrument(content_span).await {
                log::warn!("source={} event=body_read_error err={}", DEFAULT_SOURCE, e);
            }

            metrics.content_transfer_duration = content_start.elapsed();

            // Check if probe is successful
            metrics.success = metrics.status_code == config.expected_status_code;
        }
        .instrument(request_span)
        .await;
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        Self { resolver: self.resolver.clone() }
    }
}
