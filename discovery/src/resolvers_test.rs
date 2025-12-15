use std::{
    collections::HashMap,
    fs,
    net::SocketAddr,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use configuration::model::{
    Configuration, DiscoveryWrapper, RefreshInterval,
    defaults::Defaults,
    discovery::{DiscoveryApi, DiscoveryFile},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

use crate::resolver::resolve_discovery;

fn base_defaults() -> Defaults {
    Defaults {
        log_level: "info".to_string(),
        job: "zookoo".to_string(),
        service_name: "zookoo".to_string(),
        probe_location: None,
        probe_zone: None,
        self_monitoring: None,
        metric_prefix: None,
    }
}

fn temp_file_path(prefix: &str) -> PathBuf {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("zookoo_{prefix}_{now}.json"))
}

async fn start_one_shot_http_server(
    json_body: &'static str,
) -> (SocketAddr, tokio::task::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Read whatever the client sends (HTTP request). We don't need to parse it.
        let mut buf = [0u8; 4096];
        let _ = socket.read(&mut buf).await;

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            json_body.len(),
            json_body
        );

        let _ = socket.write_all(response.as_bytes()).await;
        let _ = socket.shutdown().await;
    });

    (addr, handle)
}

#[tokio::test]
async fn resolve_file_discovery_returns_some_and_reads_targets() {
    // Use HttpTarget because it can deserialize from minimal JSON containing only `url`.
    use configuration::model::target::HttpTarget;

    let json_path = temp_file_path("resolver_file_targets");
    fs::write(
        &json_path,
        r#"[
  {"url":"https://one.example"},
  {"url":"https://two.example"}
]"#,
    )
    .unwrap();

    let mut file = HashMap::new();
    file.insert(
        "dynamic".to_string(),
        DiscoveryFile {
            path: json_path.to_string_lossy().to_string(),
            labels: None,
            scrape_interval: None,
            probe_type: None,
        },
    );

    let config = Configuration {
        defaults: base_defaults(),
        probe: None,
        exporter: None,
        discovery: Some(DiscoveryWrapper { file, api: HashMap::new() }),
    };

    let discovery = resolve_discovery::<HttpTarget>("${discovery.file.dynamic}", &config)
        .await
        .expect("expected file discovery to resolve");

    let targets = discovery.get_targets().await;
    assert_eq!(targets.len(), 2);
}

#[tokio::test]
async fn resolve_file_discovery_missing_returns_none() {
    use configuration::model::target::HttpTarget;

    let config = Configuration {
        defaults: base_defaults(),
        probe: None,
        exporter: None,
        discovery: Some(DiscoveryWrapper { file: HashMap::new(), api: HashMap::new() }),
    };

    let discovery = resolve_discovery::<HttpTarget>("discovery.file.missing", &config).await;
    assert!(discovery.is_none());
}

#[tokio::test]
async fn resolve_file_discovery_invalid_reference_returns_none() {
    use configuration::model::target::HttpTarget;

    let config = Configuration {
        defaults: base_defaults(),
        probe: None,
        exporter: None,
        discovery: Some(DiscoveryWrapper { file: HashMap::new(), api: HashMap::new() }),
    };

    // Wrong prefix
    assert!(resolve_discovery::<HttpTarget>("exporter.otlp.main", &config).await.is_none());
    // Wrong discovery type
    assert!(resolve_discovery::<HttpTarget>("discovery.api.dynamic", &config).await.is_none());
    // Missing label
    assert!(resolve_discovery::<HttpTarget>("discovery.file", &config).await.is_none());
    assert!(resolve_discovery::<HttpTarget>("${discovery.file}", &config).await.is_none());
    // Extra segments are ignored by resolver (it only looks at first 3 parts) but label won't exist
    assert!(
        resolve_discovery::<HttpTarget>("discovery.file.dynamic.extra", &config).await.is_none()
    );
}

#[tokio::test]
async fn resolve_api_discovery_returns_some_and_reads_targets() {
    use configuration::model::target::HttpTarget;

    let (addr, server) = start_one_shot_http_server(
        r#"[
  {"url":"https://api-one.example"},
  {"url":"https://api-two.example"}
]"#,
    )
    .await;

    let url = format!("http://{}/targets", addr);

    let mut api = HashMap::new();
    api.insert(
        "dyn".to_string(),
        DiscoveryApi {
            url,
            headers: None,
            basic_auth: None,
            bearer: None,
            refresh_interval: RefreshInterval::S30,
        },
    );

    let config = Configuration {
        defaults: base_defaults(),
        probe: None,
        exporter: None,
        discovery: Some(DiscoveryWrapper { file: HashMap::new(), api }),
    };

    let discovery = resolve_discovery::<HttpTarget>("${discovery.api.dyn}", &config)
        .await
        .expect("expected api discovery to resolve");

    let targets = discovery.get_targets().await;
    assert_eq!(targets.len(), 2);

    // Ensure the server task finished (it only serves one request).
    let _ = server.await;
}

#[tokio::test]
async fn resolve_api_discovery_invalid_reference_returns_none() {
    use configuration::model::target::HttpTarget;

    let config = Configuration {
        defaults: base_defaults(),
        probe: None,
        exporter: None,
        discovery: Some(DiscoveryWrapper { file: HashMap::new(), api: HashMap::new() }),
    };

    // Wrong prefix
    assert!(resolve_discovery::<HttpTarget>("probe.http.main", &config).await.is_none());
    // Wrong discovery type
    assert!(resolve_discovery::<HttpTarget>("discovery.file.dynamic", &config).await.is_none());
    // Missing label
    assert!(resolve_discovery::<HttpTarget>("discovery.api", &config).await.is_none());
    assert!(resolve_discovery::<HttpTarget>("${discovery.api}", &config).await.is_none());
    // Extra segments are ignored by resolver (it only looks at first 3 parts) but label won't exist
    assert!(
        resolve_discovery::<HttpTarget>("discovery.api.dynamic.extra", &config).await.is_none()
    );
}
