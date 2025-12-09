use core::panic;
use std::{fmt::Debug, sync::Arc, time::Duration};

use async_trait::async_trait;
use configuration::model::{
    Configuration,
    target::{HttpConfiguration, IcmpConfiguration},
};
use discovery::{Discovery, file::FileDiscovery};
use exporter::Exporter;
use probe::{HttpProbe, IcmpProbe, Probe};

use crate::ExportersMap;
use crate::group_by::group_by_interval;
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
    receiver: Arc<dyn Discovery<Target = T> + Send + Sync>,
    targets: Option<Vec<T>>,
    probe: P,
    forwarder: Vec<Arc<dyn Exporter + Send + Sync>>,
    scrape_interval: Duration,
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
            receiver: self.receiver.clone(),
            targets: self.targets.clone(),
            probe: self.probe.clone(),
            forwarder: self.forwarder.clone(),
            scrape_interval: self.scrape_interval,
        }
    }
}

impl<T, P> Pipeline<T, P>
where
    T: Clone + Debug + Send + Sync + 'static,
    P: Probe<Target = T> + Send + Sync + Clone + 'static,
{
    pub fn new(
        label: String,
        probe_type: ProbeType,
        receiver: Arc<dyn Discovery<Target = T> + Send + Sync>,
        probe: P,
        forwarder: Vec<Arc<dyn Exporter + Send + Sync>>,
        scrape_interval: Duration,
    ) -> Self {
        Self { label, probe_type, receiver, targets: None, probe, forwarder, scrape_interval }
    }

    /// Discover targets from the receiver and store them
    pub fn discover_targets(&mut self) -> &Self {
        let targets = self.receiver.discover();
        if !targets.is_empty() {
            log::info!("event=targets_discovered pipeline={} count={}", self.label, targets.len());
            self.targets = Some(targets);
        } else {
            log::warn!("event=no_targets_discovered pipeline={}", self.label);
        }
        self
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
        if self.targets.is_none() {
            log::warn!("event=pipeline_skip pipeline={} reason=no_targets", self.label);
            return;
        }

        let targets = self.targets.clone().unwrap();
        log::info!(
            "event=pipeline_start pipeline={} targets={} interval_ms={}",
            self.label,
            targets.len(),
            self.scrape_interval.as_millis()
        );

        loop {
            let scrape_start = std::time::Instant::now();

            // Set targets for the probe
            self.probe.set_targets(targets.clone());

            // Scrape all targets
            log::debug!("event=scrape_cycle pipeline={}", self.label);
            self.probe.scrape().await;

            // Get collected metrics
            let metrics = self.probe.get_metrics().await;
            log::debug!(
                "event=metrics_collected pipeline={} count={} exporters={}",
                self.label,
                metrics.len(),
                self.forwarder.len()
            );

            // Forward each metric set to all exporters
            for metric_data in metrics {
                let export_data = convert_metric_data(metric_data);
                for exporter in &self.forwarder {
                    exporter.export(self.probe_type.into(), export_data.clone());
                }
            }

            // Calculate sleep time (subtract elapsed time from interval)
            let elapsed = scrape_start.elapsed();
            let sleep_duration = if elapsed < self.scrape_interval {
                self.scrape_interval - elapsed
            } else {
                log::warn!(
                    "event=scrape_slow pipeline={} elapsed_ms={} interval_ms={}",
                    self.label,
                    elapsed.as_millis(),
                    self.scrape_interval.as_millis()
                );
                Duration::ZERO
            };

            if !sleep_duration.is_zero() {
                tokio::time::sleep(sleep_duration).await;
            }
        }
    }
}

/// Builder to create pipelines from configuration
pub struct PipelineBuilder;

impl PipelineBuilder {
    /// Build all pipelines from the configuration
    /// Returns a Vec of boxed RunnablePipeline trait objects
    pub fn from_config(
        config: &Configuration,
        exporters: ExportersMap,
    ) -> Vec<Box<dyn RunnablePipeline>> {
        let mut pipelines: Vec<Box<dyn RunnablePipeline>> = Vec::new();

        // Build HTTP pipelines (grouped by scrape interval)
        if let Some(ref probe_wrapper) = config.probe {
            for (label, http_config) in &probe_wrapper.http {
                let http_pipelines =
                    Self::build_http_pipelines(label, http_config, config, &exporters);
                for pipeline in http_pipelines {
                    pipelines.push(Box::new(pipeline));
                }
            }

            // Build ICMP pipelines (grouped by scrape interval)
            for (label, icmp_config) in &probe_wrapper.icmp {
                let icmp_pipelines =
                    Self::build_icmp_pipelines(label, icmp_config, config, &exporters);
                for pipeline in icmp_pipelines {
                    pipelines.push(Box::new(pipeline));
                }
            }
        }

        pipelines
    }

    /// Build HTTP pipelines - one per unique scrape interval
    fn build_http_pipelines(
        label: &str,
        http_config: &HttpConfiguration,
        config: &Configuration,
        exporters: &ExportersMap,
    ) -> Vec<Pipeline<configuration::model::target::HttpTarget, HttpProbe>> {
        use configuration::model::target::HttpTarget;

        let mut pipelines = Vec::new();

        // Resolve targets
        let targets: Vec<HttpTarget> = if let Some(ref target_from) = http_config.target_from {
            if let Some(file_discovery) =
                Self::resolve_file_discovery::<HttpTarget>(target_from, config)
            {
                file_discovery.discover()
            } else {
                log::error!(
                    "event=error pipeline={} msg=could_not_resolve_target_from target_from={}",
                    label,
                    target_from
                );
                panic!("Could not resolve target_from: {}", target_from);
            }
        } else if let Some(ref targets) = http_config.targets {
            targets.clone()
        } else {
            log::error!("event=error pipeline={} msg=no_targets_or_target_from_specified", label);
            return pipelines;
        };

        if targets.is_empty() {
            log::warn!("event=no_targets_discovered pipeline={}", label);
            return pipelines;
        }

        // Group targets by scrape interval
        let grouped = group_by_interval(&targets, http_config.scrape_interval);
        let group_count = grouped.len();
        let resolved_exporters = Self::resolve_exporters(&http_config.forward_to, exporters);

        for (interval, group_targets) in grouped {
            let pipeline_label = if group_count > 1 {
                format!("{}@{:?}", label, interval)
            } else {
                label.to_string()
            };

            log::info!(
                "event=pipeline_created pipeline={} targets={} interval={:?}",
                pipeline_label,
                group_targets.len(),
                interval
            );

            let discovery = Arc::new(StaticDiscovery::new(group_targets));
            let mut pipeline = Pipeline::new(
                pipeline_label,
                ProbeType::Http,
                discovery,
                HttpProbe::init(),
                resolved_exporters.clone(),
                interval.to_duration(),
            );
            pipeline.discover_targets();
            pipelines.push(pipeline);
        }

        pipelines
    }

    /// Build ICMP pipelines - one per unique scrape interval
    fn build_icmp_pipelines(
        label: &str,
        icmp_config: &IcmpConfiguration,
        config: &Configuration,
        exporters: &ExportersMap,
    ) -> Vec<Pipeline<configuration::model::target::IcmpTarget, IcmpProbe>> {
        use configuration::model::target::IcmpTarget;

        let mut pipelines = Vec::new();

        // Resolve targets
        let targets: Vec<IcmpTarget> = if let Some(ref target_from) = icmp_config.target_from {
            if let Some(file_discovery) =
                Self::resolve_file_discovery::<IcmpTarget>(target_from, config)
            {
                file_discovery.discover()
            } else {
                log::error!(
                    "event=error pipeline={} msg=could_not_resolve_target_from target_from={}",
                    label,
                    target_from
                );
                return pipelines;
            }
        } else if let Some(ref targets) = icmp_config.targets {
            targets.clone()
        } else {
            log::error!("event=error pipeline={} msg=no_targets_or_target_from_specified", label);
            return pipelines;
        };

        if targets.is_empty() {
            log::warn!("event=no_targets_discovered pipeline={}", label);
            return pipelines;
        }

        // Group targets by scrape interval
        let grouped = group_by_interval(&targets, icmp_config.scrape_interval);
        let group_count = grouped.len();
        let resolved_exporters = Self::resolve_exporters(&icmp_config.forward_to, exporters);

        for (interval, group_targets) in grouped {
            let pipeline_label = if group_count > 1 {
                format!("{}@{:?}", label, interval)
            } else {
                label.to_string()
            };

            log::info!(
                "event=pipeline_created pipeline={} targets={} interval={:?}",
                pipeline_label,
                group_targets.len(),
                interval
            );

            let discovery = Arc::new(StaticDiscovery::new(group_targets));
            let mut pipeline = Pipeline::new(
                pipeline_label,
                ProbeType::Icmp,
                discovery,
                IcmpProbe::init(),
                resolved_exporters.clone(),
                interval.to_duration(),
            );
            pipeline.discover_targets();
            pipelines.push(pipeline);
        }

        pipelines
    }

    /// Resolve a file discovery reference like "discovery.file.json_targets" or "${discovery.file.json_targets}"
    fn resolve_file_discovery<T>(
        reference: &str,
        config: &Configuration,
    ) -> Option<FileDiscovery<T>>
    where
        T: Clone + std::fmt::Debug + Send + Sync + serde::de::DeserializeOwned + 'static,
    {
        // Strip ${} wrapper if present
        let reference =
            reference.strip_prefix("${").and_then(|s| s.strip_suffix("}")).unwrap_or(reference);

        let parts: Vec<&str> = reference.split('.').collect();
        match (parts.first(), parts.get(1), parts.get(2)) {
            (Some(&"discovery"), Some(&"file"), Some(label)) => {
                if let Some(ref discovery_wrapper) = config.discovery
                    && let Some(file_config) = discovery_wrapper.file.get(*label)
                {
                    // Use first path for now
                    if let Some(path) = file_config.path.first() {
                        return Some(FileDiscovery::new(path));
                    }
                }
                None
            }
            _ => None,
        }
    }

    /// Resolve exporters from forward_to references
    /// References can be like "exporter.otlp.main" or "${exporter.otlp.main}"
    fn resolve_exporters(
        forward_to: &[String],
        all_exporters: &ExportersMap,
    ) -> Vec<Arc<dyn Exporter + Send + Sync>> {
        if forward_to.is_empty() {
            // If no forward_to specified, throw an error and panic
            log::error!("event=error msg=no_forward_to_specified_for_exporters");
            log::error!("INVALID CONFIGURATION");
            log::error!(
                "Unrecoverable error: No forward_to specified for exporters ! Key forward_to is mandatory in probe configuration."
            );
            panic!("No forward_to specified for exporters");
        }

        let mut resolved = Vec::new();
        for reference in forward_to {
            // Strip ${} wrapper if present
            let key =
                reference.strip_prefix("${").and_then(|s| s.strip_suffix("}")).unwrap_or(reference);

            if let Some(exporter) = all_exporters.get(key) {
                resolved.push(exporter.clone());
                log::debug!("event=exporter_resolved reference={} key={}", reference, key);
            } else {
                log::error!(
                    "event=exporter_not_found reference={} available={:?}",
                    reference,
                    all_exporters.keys().collect::<Vec<_>>()
                );
                log::error!("INVALID CONFIGURATION");
                panic!("Exporter not found for reference: {}", reference);
            }
        }

        resolved
    }
}

/// A simple discovery that returns static targets
struct StaticDiscovery<T: Clone + std::fmt::Debug + Send + Sync + 'static> {
    targets: Vec<T>,
}

impl<T: Clone + std::fmt::Debug + Send + Sync + 'static> StaticDiscovery<T> {
    fn new(targets: Vec<T>) -> Self {
        Self { targets }
    }
}

impl<T: Clone + std::fmt::Debug + Send + Sync + 'static> Discovery for StaticDiscovery<T> {
    type Target = T;

    fn discover(&self) -> Vec<Self::Target> {
        self.targets.clone()
    }

    fn update(&self) {
        // Static discovery doesn't update
    }
}
