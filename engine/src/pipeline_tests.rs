use std::sync::{
    Arc,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::{RwLock, watch};

use discovery::Discovery;
use probe::Probe;

use crate::pipeline::{Pipeline, RunnablePipeline};
use crate::types::ProbeType;

#[derive(Debug)]
struct MockDiscovery {
    targets: Arc<RwLock<Vec<String>>>,
    version: AtomicU64,
    get_targets_calls: AtomicUsize,
    tx: watch::Sender<u64>,
    rx: watch::Receiver<u64>,
}

impl MockDiscovery {
    fn new(initial: Vec<String>) -> Self {
        let (tx, rx) = watch::channel(0u64);
        Self {
            targets: Arc::new(RwLock::new(initial)),
            version: AtomicU64::new(0),
            get_targets_calls: AtomicUsize::new(0),
            tx,
            rx,
        }
    }

    async fn set_targets_and_notify(&self, next: Vec<String>) {
        self.targets.write().await.clone_from(&next);
        self.version.fetch_add(1, Ordering::Relaxed);
        let _ = self.tx.send(self.version());
    }

    fn get_targets_calls(&self) -> usize {
        self.get_targets_calls.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl discovery::Discovery for MockDiscovery {
    type Target = String;

    async fn discover(&self) {}

    fn update(&self) {}

    fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    fn subscribe(&self) -> Option<watch::Receiver<u64>> {
        Some(self.rx.clone())
    }

    async fn get_targets(&self) -> Vec<Self::Target> {
        self.get_targets_calls.fetch_add(1, Ordering::Relaxed);
        self.targets.read().await.clone()
    }
}

#[derive(Clone)]
struct NoopProbe {
    _seen: Arc<tokio::sync::Mutex<Vec<Vec<String>>>>,
}

impl probe::Probe for NoopProbe {
    type Target = String;

    fn init() -> Self {
        Self { _seen: Arc::new(tokio::sync::Mutex::new(Vec::new())) }
    }

    fn set_targets(&mut self, data: Vec<Self::Target>) {
        let seen = self._seen.clone();
        tokio::spawn(async move {
            seen.lock().await.push(data);
        });
    }

    async fn scrape(&self) {}

    async fn get_metrics(&self) -> Vec<probe::MetricData> {
        Vec::new()
    }
}

#[tokio::test]
async fn pipeline_processes_discovery_updates_even_when_starting_empty() {
    let discovery = Arc::new(MockDiscovery::new(Vec::new()));
    let discovery_dyn: Arc<dyn discovery::Discovery<Target = String> + Send + Sync> =
        discovery.clone();

    let mut pipeline = Pipeline::new(
        "p".to_string(),
        ProbeType::Http,
        Some(discovery_dyn),
        NoopProbe::init(),
        Vec::new(),
        Duration::from_secs(3600),
    );

    // Start with empty targets so the first scrape cycle would previously block on sleep.
    pipeline.targets = Some(Vec::new());

    let handle = tokio::spawn(async move {
        let mut p = pipeline;
        p.run().await;
    });

    // Let the pipeline start, run its initial scrape attempt and initial refresh.
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;

    let calls_after_start = discovery.get_targets_calls();
    assert!(calls_after_start >= 1, "pipeline should refresh targets at startup");

    // Trigger a discovery update without advancing time; pipeline must react immediately.
    discovery.set_targets_and_notify(vec!["https://new".to_string()]).await;
    tokio::task::yield_now().await;
    tokio::task::yield_now().await;

    assert!(
        discovery.get_targets_calls() > calls_after_start,
        "pipeline should refresh targets in response to discovery update"
    );

    handle.abort();
}
