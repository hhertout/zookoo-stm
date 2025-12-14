use std::{fmt::Debug, future::pending, sync::Arc, time::Duration};

use async_trait::async_trait;
use discovery::Discovery;
use exporter::Exporter;
use probe::Probe;
use tokio::sync::watch;

use crate::types::{ProbeType, convert_metric_data};

/// A runnable pipeline that can be spawned as a task
#[async_trait]
pub trait RunnablePipeline: Send + Sync {
    async fn run(&mut self);
    fn label(&self) -> &str;
}

pub(crate) struct Pipeline<T, P>
where
    T: Clone + std::fmt::Debug + Send + Sync + 'static,
    P: probe::Probe<Target = T> + Send + Sync + Clone + 'static,
{
    label: String,
    probe_type: ProbeType,
    discovery: Option<Arc<dyn Discovery<Target = T> + Send + Sync>>,
    pub targets: Option<Vec<T>>,
    probe: P,
    forwarder: Vec<Arc<dyn Exporter + Send + Sync>>,
    scrape_interval: Duration,
    discovery_updates: Option<watch::Receiver<u64>>,
}

impl<T, P> Clone for Pipeline<T, P>
where
    T: Clone + Debug + Send + Sync + 'static,
    P: Probe<Target = T> + Send + Sync + Clone + 'static,
{
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            probe_type: self.probe_type,
            discovery: self.discovery.clone(),
            targets: self.targets.clone(),
            probe: self.probe.clone(),
            forwarder: self.forwarder.clone(),
            scrape_interval: self.scrape_interval,
            discovery_updates: self.discovery_updates.clone(),
        }
    }
}

impl<T, P> Pipeline<T, P>
where
    T: Clone + Debug + Send + Sync + 'static,
    P: Probe<Target = T> + Send + Sync + Clone + 'static,
{
    fn start_discovery_updates(&mut self) {
        let Some(discovery) = &self.discovery else {
            return;
        };

        let _ = discovery.update();
        self.discovery_updates = discovery.subscribe();
    }

    async fn refresh_targets_from_discovery(&mut self) {
        let Some(discovery) = &self.discovery else {
            return;
        };

        // No per-target scrape interval overrides: a pipeline always runs at its probe interval.
        self.targets = Some(discovery.get_targets().await);
    }

    async fn scrape_once(&mut self) {
        let targets = self.targets.clone().unwrap_or_default();
        if targets.is_empty() {
            log::warn!(
                "event=pipeline_no_targets pipeline={} recovery=retry_next_tick",
                self.label,
            );
            tokio::time::sleep(self.scrape_interval).await;
            return;
        }

        self.probe.set_targets(targets);
        log::debug!("event=scrape_cycle pipeline={}", self.label);

        let scrape_start = std::time::Instant::now();
        self.probe.scrape().await;

        let metrics = self.probe.get_metrics().await;
        log::debug!(
            "event=metrics_collected pipeline={} count={} exporters={}",
            self.label,
            metrics.len(),
            self.forwarder.len()
        );

        for metric_data in metrics {
            let export_data = convert_metric_data(metric_data);
            for exporter in &self.forwarder {
                exporter.export(self.probe_type.into(), export_data.clone());
            }
        }

        let elapsed = scrape_start.elapsed();
        if elapsed > self.scrape_interval {
            log::warn!(
                "event=scrape_slow pipeline={} elapsed_ms={} interval_ms={}",
                self.label,
                elapsed.as_millis(),
                self.scrape_interval.as_millis()
            );
        }
    }

    pub fn new(
        label: String,
        probe_type: ProbeType,
        discovery: Option<Arc<dyn Discovery<Target = T> + Send + Sync>>,
        probe: P,
        forwarder: Vec<Arc<dyn Exporter + Send + Sync>>,
        scrape_interval: Duration,
    ) -> Self {
        Self {
            label,
            probe_type,
            discovery,
            targets: None,
            probe,
            forwarder,
            scrape_interval,
            discovery_updates: None,
        }
    }
}

#[async_trait]
impl<T, P> RunnablePipeline for Pipeline<T, P>
where
    T: Clone + Debug + Send + Sync + 'static,
    P: Probe<Target = T> + Send + Sync + Clone + 'static,
{
    fn label(&self) -> &str {
        &self.label
    }

    /// Run the pipeline loop: scrape -> collect metrics -> export
    /// This runs indefinitely until the task is cancelled
    async fn run(&mut self) {
        if self.targets.is_none() && self.discovery.is_none() {
            log::warn!("event=pipeline_skip pipeline={} reason=no_targets", self.label);
            return;
        }

        self.start_discovery_updates();
        self.refresh_targets_from_discovery().await;

        let mut ticker = tokio::time::interval(self.scrape_interval);

        loop {
            tokio::select! {
                _ = ticker.tick() => self.scrape_once().await,
                _ = async {
                    match &mut self.discovery_updates {
                        Some(rx) => { let _ = rx.changed().await; }
                        None => pending::<()>().await,
                    }
                } => {
                    self.refresh_targets_from_discovery().await;
                }
            }
        }
    }
}
