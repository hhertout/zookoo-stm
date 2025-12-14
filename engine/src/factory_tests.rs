use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use configuration::model::{
    Configuration, DiscoveryWrapper, ProbeWrapper,
    defaults::Defaults,
    discovery::DiscoveryFile,
    scrape_interval::ScrapeInterval,
    target::{HttpConfiguration, HttpTarget, IcmpConfiguration, IcmpTarget},
};
use exporter::{Exporter, MetricData};

use crate::{ExportersMap, factory::PipelineBuilder, types::ProbeType};

#[derive(Default)]
struct NoopExporter;

impl Exporter for NoopExporter {
    fn build(_config: &Configuration, _exporters: &mut exporter::types::ExportersMap)
    where
        Self: Sized,
    {
    }

    fn export(&self, _probe_type: exporter::types::ProbeType, _metric_data: MetricData) {}
}

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

fn http_target(url: &str) -> HttpTarget {
    HttpTarget {
        method: "GET".to_string(),
        url: url.to_string(),
        expected_status_code: 200,
        headers: None,
        labels: None,
        auth: None,
        timeout_sec: 15,
        follow_redirect: false,
        skip_tls: false,
    }
}

fn icmp_target(ipv4: &str) -> IcmpTarget {
    IcmpTarget { ipv4: Some(ipv4.to_string()), fqdn: None, labels: None, timeout_sec: 15 }
}

fn exporters_with(reference: &str) -> ExportersMap {
    let mut exporters: ExportersMap = HashMap::new();
    exporters.insert(reference.to_string(), Arc::new(NoopExporter::default()));
    exporters
}

#[tokio::test]
async fn build_http_pipelines_direct_targets_creates_pipeline() {
    let forward_ref = "exporter.otlp.main";

    let http_config = HttpConfiguration {
        targets: Some(vec![http_target("https://a.example"), http_target("https://b.example")]),
        target_from: None,
        forward_to: vec![forward_ref.to_string()],
        scrape_interval: ScrapeInterval::S30,
    };

    let config =
        Configuration { defaults: base_defaults(), probe: None, exporter: None, discovery: None };

    let pipelines = PipelineBuilder::build_pipelines::<HttpTarget, probe::HttpProbe, _>(
        "p_http",
        &http_config,
        &config,
        &exporters_with(forward_ref),
        ProbeType::Http,
        <probe::HttpProbe as probe::Probe>::init,
    )
    .await;

    assert_eq!(pipelines.len(), 1);
    assert_eq!(pipelines[0].targets.as_ref().unwrap().len(), 2);
}

#[tokio::test]
async fn build_http_pipelines_target_from_missing_discovery_returns_empty() {
    let forward_ref = "exporter.otlp.main";

    let http_config = HttpConfiguration {
        targets: None,
        target_from: Some("discovery.file.missing".to_string()),
        forward_to: vec![forward_ref.to_string()],
        scrape_interval: ScrapeInterval::S30,
    };

    let config =
        Configuration { defaults: base_defaults(), probe: None, exporter: None, discovery: None };

    let pipelines = PipelineBuilder::build_pipelines::<HttpTarget, probe::HttpProbe, _>(
        "p_http",
        &http_config,
        &config,
        &exporters_with(forward_ref),
        ProbeType::Http,
        <probe::HttpProbe as probe::Probe>::init,
    )
    .await;

    assert!(pipelines.is_empty());
}

#[tokio::test]
async fn build_http_pipelines_target_from_file_uses_file_targets() {
    let forward_ref = "exporter.otlp.main";

    let json_path = temp_file_path("http_targets");
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

    let http_config = HttpConfiguration {
        targets: None,
        target_from: Some("discovery.file.dynamic".to_string()),
        forward_to: vec![forward_ref.to_string()],
        scrape_interval: ScrapeInterval::S30,
    };

    let pipelines = PipelineBuilder::build_pipelines::<HttpTarget, probe::HttpProbe, _>(
        "p_http",
        &http_config,
        &config,
        &exporters_with(forward_ref),
        ProbeType::Http,
        <probe::HttpProbe as probe::Probe>::init,
    )
    .await;

    assert_eq!(pipelines.len(), 1);
    assert_eq!(pipelines[0].targets.as_ref().unwrap().len(), 2);
}

#[tokio::test]
async fn build_http_pipelines_direct_targets_override_target_from() {
    let forward_ref = "exporter.otlp.main";

    let json_path = temp_file_path("http_targets_override");
    fs::write(
        &json_path,
        r#"[
  {"url":"https://from-file.example"}
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

    let http_config = HttpConfiguration {
        // Direct targets should win.
        targets: Some(vec![http_target("https://direct.example")]),
        target_from: Some("discovery.file.dynamic".to_string()),
        forward_to: vec![forward_ref.to_string()],
        scrape_interval: ScrapeInterval::S30,
    };

    let pipelines = PipelineBuilder::build_pipelines::<HttpTarget, probe::HttpProbe, _>(
        "p_http",
        &http_config,
        &config,
        &exporters_with(forward_ref),
        ProbeType::Http,
        <probe::HttpProbe as probe::Probe>::init,
    )
    .await;

    assert_eq!(pipelines.len(), 1);
    assert_eq!(pipelines[0].targets.as_ref().unwrap().len(), 1);
    assert_eq!(pipelines[0].targets.as_ref().unwrap()[0].url, "https://direct.example");
}

#[tokio::test]
async fn from_config_builds_http_and_icmp_pipelines() {
    let forward_ref = "exporter.otlp.main";

    let mut http = HashMap::new();
    http.insert(
        "h".to_string(),
        HttpConfiguration {
            targets: Some(vec![http_target("https://x.example")]),
            target_from: None,
            forward_to: vec![forward_ref.to_string()],
            scrape_interval: ScrapeInterval::S30,
        },
    );

    let mut icmp = HashMap::new();
    icmp.insert(
        "i".to_string(),
        IcmpConfiguration {
            targets: Some(vec![icmp_target("1.1.1.1")]),
            target_from: None,
            forward_to: vec![forward_ref.to_string()],
            scrape_interval: ScrapeInterval::S30,
        },
    );

    let config = Configuration {
        defaults: base_defaults(),
        probe: Some(ProbeWrapper { http, icmp }),
        exporter: None,
        discovery: None,
    };

    let pipelines = PipelineBuilder::from_config(&config, exporters_with(forward_ref)).await;
    assert_eq!(pipelines.len(), 2);

    let labels: HashSet<String> = pipelines.into_iter().map(|p| p.label().to_string()).collect();
    assert!(labels.contains("h"));
    assert!(labels.contains("i"));
}
