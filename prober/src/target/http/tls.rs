use chrono::TimeZone;
use chrono::{NaiveDateTime, Utc};
use native_tls::TlsConnector as NativeTls;
use openssl::x509::X509NameRef;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::net::{TcpStream, lookup_host};
use tokio::time::timeout;
use tokio_native_tls::TlsConnector;

#[derive(Debug, Clone, Serialize)]
pub struct TlsMetrics {
    pub valid: u8,
    pub issuer: String,
    pub subject: String,
    pub algo: String,
    pub duration: Duration,
    pub handshake_duration: Duration,
    pub cert_expiration_date: Option<i64>,
    pub cert_begin_date: Option<i64>,
    pub version: i32,
}

impl TlsMetrics {
    pub fn invalid() -> Self {
        TlsMetrics {
            valid: 0,
            issuer: String::from("unknow"),
            subject: String::from("unknow"),
            algo: String::from("unknow"),
            duration: Duration::from_secs(15),
            handshake_duration: Duration::from_secs(15),
            cert_expiration_date: None,
            cert_begin_date: None,
            version: 0,
        }
    }

    pub fn to_labels(&self) -> HashMap<String, String> {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert(
            String::from("issuer"),
            self.issuer.clone().replace(" ", "").replace("=", ":"),
        );
        map.insert(
            String::from("subject"),
            self.subject.clone().replace(" ", "").replace("=", ":"),
        );
        map.insert(
            String::from("algorithm"),
            self.algo.clone().replace(" ", "").replace("=", ":"),
        );

        return map;
    }

    pub fn to_logfmt(&self) -> String {
        format!(
            "valid={} issuer={} subject={} algo={} duration={:?} handshake_duration={:?} cert_begin_date_ts={} cert_expiration_date_ts={} version={}",
            self.valid,
            self.issuer.replace(" ", "").replace("=", ":"),
            self.subject.replace(" ", "").replace("=", ":"),
            self.algo.replace(" ", ""),
            self.duration.as_millis(),
            self.handshake_duration.as_millis(),
            self.cert_expiration_date
                .map_or("unknow".to_string(), |v| v.to_string()),
            self.cert_begin_date
                .map_or("unknow".to_string(), |v| v.to_string()),
            self.version
        )
    }
}

pub async fn inspect_tls(url: &str) -> Result<TlsMetrics, Box<dyn std::error::Error>> {
    let parsed_url = url::Url::parse(url)?;
    let host = parsed_url.host_str().ok_or("Invalid host in URL")?;

    let port = 443;

    let start = Instant::now();

    let addr = lookup_host((host, port))
        .await?
        .find(|addr| addr.is_ipv4())
        .ok_or("no_ipv4")?;

    let stream = timeout(Duration::from_secs(15), TcpStream::connect(addr))
        .await
        .map_err(|_| "timeout")??;

    let dns_name = host;
    let connector = NativeTls::new()?;
    let connector = TlsConnector::from(connector);

    let tls_start = Instant::now();
    let tls_stream = connector.connect(dns_name, stream).await?;
    let tls_duration = tls_start.elapsed();

    let cert = tls_stream
        .get_ref()
        .peer_certificate()?
        .ok_or("no certificate")?;
    let cert = cert.to_der()?;

    let x509 = openssl::x509::X509::from_der(&cert)?;

    let total = start.elapsed();

    let algo = x509.signature_algorithm().object();
    let algo_name = algo.nid().long_name().unwrap_or("unknown");
    let algo_code = algo.nid().as_raw();
    let algo = format!("{} ({})", algo_name, algo_code).to_owned();

    Ok(TlsMetrics {
        valid: 1,
        issuer: x509_name_to_string(&x509.issuer_name()),
        subject: x509_name_to_string(&x509.subject_name()),
        algo,
        version: x509.version(),
        duration: total,
        handshake_duration: tls_duration,
        cert_begin_date: parse_ssl_date_to_timestamp(&x509.not_before().to_string()),
        cert_expiration_date: parse_ssl_date_to_timestamp(&x509.not_after().to_string()),
    })
}

fn parse_ssl_date_to_timestamp(date_str: &str) -> Option<i64> {
    let trimmed = date_str.strip_suffix(" GMT")?;
    let format = "%b %d %H:%M:%S %Y";
    NaiveDateTime::parse_from_str(trimmed, format)
        .ok()
        .map(|ndt| Utc.from_utc_datetime(&ndt).timestamp())
}

fn x509_name_to_string(name: &X509NameRef) -> String {
    name.entries()
        .map(|e| {
            let key = e.object().nid().short_name().unwrap_or("UNKNOW");
            let value = e
                .data()
                .as_utf8()
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "???".to_string());
            format!("{}={}", key, value)
        })
        .collect::<Vec<_>>()
        .join(", ")
}
