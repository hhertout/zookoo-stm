//! TLS Connector
//!
//! Provides TLS connection handling with certificate inspection using rustls.

use std::sync::Arc;
use std::time::{Duration, Instant};

use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use tracing::instrument;
use x509_parser::prelude::*;

/// TLS certificate information extracted during handshake
#[derive(Debug, Clone)]
pub struct CertInfo {
    /// Certificate subject common name
    pub subject: Option<String>,
    /// Certificate issuer common name
    pub issuer: Option<String>,
    /// Certificate validity start (Unix timestamp)
    pub not_before: Option<i64>,
    /// Certificate validity end (Unix timestamp)
    pub not_after: Option<i64>,
    /// TLS protocol version used
    pub tls_version: String,
}

impl CertInfo {
    pub fn empty() -> Self {
        Self {
            subject: None,
            issuer: None,
            not_before: None,
            not_after: None,
            tls_version: String::new(),
        }
    }
}

/// Result of a TLS handshake operation
pub struct TlsResult<S> {
    /// The TLS stream
    pub stream: TlsStream<S>,
    /// Handshake duration
    pub handshake_duration: Duration,
    /// Certificate information
    pub cert_info: CertInfo,
}

/// TLS Connector with certificate inspection capabilities
pub struct TlsHandler {
    connector: TlsConnector,
    /// Whether to skip certificate verification
    skip_verify: bool,
}

impl TlsHandler {
    /// Create a new TLS handler with system root certificates
    #[instrument(name = "tls_handler_new", fields(skip_verify = skip_verify))]
    pub fn new(skip_verify: bool) -> Result<Self, String> {
        // Install the default crypto provider if not already installed
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

        let root_store = if skip_verify {
            // Empty root store for skip verification mode
            RootCertStore::empty()
        } else {
            // Load system root certificates
            let mut roots = RootCertStore::empty();
            roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            roots
        };

        let config = if skip_verify {
            // Create config that skips verification
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(NoVerifier))
                .with_no_client_auth()
        } else {
            ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth()
        };

        let connector = TlsConnector::from(Arc::new(config));

        Ok(Self { connector, skip_verify })
    }

    /// Perform TLS handshake on an existing TCP stream
    ///
    /// # Arguments
    /// * `stream` - The TCP stream to upgrade to TLS
    /// * `server_name` - The server name for SNI
    ///
    /// # Returns
    /// * `Ok(TlsResult)` - Contains TLS stream, timing, and cert info
    /// * `Err(String)` - Error message if handshake fails
    #[instrument(name = "tls_handshake_execute", skip(self, stream), fields(server_name = %server_name))]
    pub async fn handshake(
        &self,
        stream: TcpStream,
        server_name: &str,
    ) -> Result<TlsResult<TcpStream>, String> {
        let start = Instant::now();

        let server_name = ServerName::try_from(server_name.to_string())
            .map_err(|e| format!("Invalid server name: {}", e))?;

        let tls_stream = self
            .connector
            .connect(server_name, stream)
            .await
            .map_err(|e| format!("TLS handshake failed: {}", e))?;

        let handshake_duration = start.elapsed();

        // Extract certificate information
        let cert_info = self.extract_cert_info(&tls_stream);

        Ok(TlsResult { stream: tls_stream, handshake_duration, cert_info })
    }

    /// Extract certificate information from a TLS stream
    #[instrument(name = "tls_extract_cert_info", skip(self, stream))]
    fn extract_cert_info<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        stream: &TlsStream<S>,
    ) -> CertInfo {
        let (_, conn) = stream.get_ref();

        let tls_version = match conn.protocol_version() {
            Some(rustls::ProtocolVersion::TLSv1_2) => "TLSv1.2".to_string(),
            Some(rustls::ProtocolVersion::TLSv1_3) => "TLSv1.3".to_string(),
            Some(v) => format!("{:?}", v),
            None => "unknown".to_string(),
        };

        // Try to get peer certificates
        let certs = match conn.peer_certificates() {
            Some(certs) if !certs.is_empty() => certs,
            _ => {
                return CertInfo { tls_version, ..CertInfo::empty() };
            }
        };

        // Parse the first certificate (server cert)
        let cert_der = &certs[0];
        let parsed = match X509Certificate::from_der(cert_der.as_ref()) {
            Ok((_, cert)) => cert,
            Err(_) => {
                return CertInfo { tls_version, ..CertInfo::empty() };
            }
        };

        // Extract subject CN
        let subject = parsed
            .subject()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .map(|s| s.to_string());

        // Extract issuer CN
        let issuer = parsed
            .issuer()
            .iter_common_name()
            .next()
            .and_then(|cn| cn.as_str().ok())
            .map(|s| s.to_string());

        // Extract validity dates
        let not_before = Some(parsed.validity().not_before.timestamp());
        let not_after = Some(parsed.validity().not_after.timestamp());

        CertInfo { subject, issuer, not_before, not_after, tls_version }
    }
}

impl Clone for TlsHandler {
    fn clone(&self) -> Self {
        Self::new(self.skip_verify).expect("Failed to clone TlsHandler")
    }
}

/// Custom certificate verifier that accepts all certificates (for skip_tls mode)
#[derive(Debug)]
struct NoVerifier;

impl rustls::client::danger::ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::pki_types::CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
