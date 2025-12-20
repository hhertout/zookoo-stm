//! DNS Resolver
//!
//! Provides async DNS resolution with timing metrics using hickory-resolver.

use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use hickory_resolver::Resolver;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::name_server::TokioConnectionProvider;

use tracing::instrument;

/// Result of a DNS lookup operation
#[derive(Debug, Clone)]
pub struct DnsResult {
    /// Resolved IP addresses
    pub addresses: Vec<IpAddr>,
    /// Time taken for DNS resolution
    pub duration: Duration,
}

/// DNS Resolver with caching and timing
pub struct DnsResolver {
    resolver: Arc<Resolver<TokioConnectionProvider>>,
}

impl DnsResolver {
    /// Create a new DNS resolver with system configuration
    #[instrument(name = "dns_resolver_new")]
    pub fn new() -> Self {
        let resolver = Resolver::builder_with_config(
            ResolverConfig::default(),
            TokioConnectionProvider::default(),
        )
        .with_options(ResolverOpts::default())
        .build();

        Self { resolver: Arc::new(resolver) }
    }

    /// Create a new DNS resolver with custom configuration
    pub fn with_config(config: ResolverConfig, opts: ResolverOpts) -> Self {
        let resolver = Resolver::builder_with_config(config, TokioConnectionProvider::default())
            .with_options(opts)
            .build();

        Self { resolver: Arc::new(resolver) }
    }

    /// Resolve a hostname to IP addresses with timing
    ///
    /// # Arguments
    /// * `host` - The hostname to resolve (without scheme or port)
    ///
    /// # Returns
    /// * `Ok(DnsResult)` - Contains resolved addresses and timing
    /// * `Err(String)` - Error message if resolution fails
    #[instrument(name = "dns_resolve", skip(self), fields(host = %host))]
    pub async fn resolve(&self, host: &str) -> Result<DnsResult, String> {
        let start = Instant::now();

        let response = self
            .resolver
            .lookup_ip(host)
            .await
            .map_err(|e| format!("DNS resolution failed for {}: {}", host, e))?;

        let duration = start.elapsed();

        let addresses: Vec<IpAddr> = response.iter().collect();

        if addresses.is_empty() {
            return Err(format!("No addresses found for {}", host));
        }

        Ok(DnsResult { addresses, duration })
    }

    /// Resolve and return the first IPv4 address (preferred for compatibility)
    #[instrument(name = "dns_resolve_first_ipv4", skip(self), fields(host = %host))]
    pub async fn resolve_first_ipv4(&self, host: &str) -> Result<(IpAddr, Duration), String> {
        let result = self.resolve(host).await?;

        // Prefer IPv4 for compatibility
        let addr = result
            .addresses
            .iter()
            .find(|a| a.is_ipv4())
            .or_else(|| result.addresses.first())
            .copied()
            .ok_or_else(|| format!("No addresses found for {}", host))?;

        Ok((addr, result.duration))
    }
}

impl Default for DnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DnsResolver {
    fn clone(&self) -> Self {
        Self { resolver: Arc::clone(&self.resolver) }
    }
}

/// Extract hostname from a URL string
pub fn extract_host(url: &str) -> Result<String, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;

    parsed.host_str().map(|s| s.to_string()).ok_or_else(|| "URL has no host".to_string())
}

/// Extract port from a URL string (returns default port based on scheme)
pub fn extract_port(url: &str) -> Result<u16, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("Invalid URL: {}", e))?;

    Ok(parsed.port_or_known_default().unwrap_or(80))
}
