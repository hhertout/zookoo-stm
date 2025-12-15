use std::sync::Arc;

use configuration::model::{
    Configuration, ScrapeInterval,
    target::{HttpConfiguration, HttpTarget, IcmpConfiguration, IcmpTarget},
};
use discovery::{Discovery, resolver::resolve_discovery};
use exporter::resolvers::resolve_exporters;
use probe::{HttpProbe, IcmpProbe, Probe};

use crate::{
    ExportersMap,
    pipeline::{Pipeline, RunnablePipeline},
    types::ProbeType,
};

/// Builder to create pipelines from configuration
pub struct PipelineBuilder;

pub trait PipelineConfig<T> {
    fn targets(&self) -> &Option<Vec<T>>;
    fn target_from(&self) -> &Option<String>;
    fn forward_to(&self) -> &[String];
    fn scrape_interval(&self) -> ScrapeInterval;
}

impl PipelineConfig<HttpTarget> for HttpConfiguration {
    fn targets(&self) -> &Option<Vec<HttpTarget>> {
        &self.targets
    }

    fn target_from(&self) -> &Option<String> {
        &self.target_from
    }

    fn forward_to(&self) -> &[String] {
        &self.forward_to
    }

    fn scrape_interval(&self) -> ScrapeInterval {
        self.scrape_interval
    }
}

impl PipelineConfig<IcmpTarget> for IcmpConfiguration {
    fn targets(&self) -> &Option<Vec<IcmpTarget>> {
        &self.targets
    }

    fn target_from(&self) -> &Option<String> {
        &self.target_from
    }

    fn forward_to(&self) -> &[String] {
        &self.forward_to
    }

    fn scrape_interval(&self) -> ScrapeInterval {
        self.scrape_interval
    }
}

impl PipelineBuilder {
    pub async fn build_pipelines<T, P, C>(
        label: &str,
        probe_config: &C,
        config: &Configuration,
        exporters: &ExportersMap,
        probe_type: ProbeType,
        probe_init: fn() -> P,
    ) -> Vec<Pipeline<T, P>>
    where
        T: Clone + std::fmt::Debug + Send + Sync + serde::de::DeserializeOwned + 'static, // Target Type
        P: Probe<Target = T> + Send + Sync + Clone + 'static, // ProbeType
        C: PipelineConfig<T>,                                 // Probe Configuration Type
    {
        let mut pipelines = Vec::new();

        // Resolve discovery
        let mut targets: Vec<T> = Vec::new();
        let mut discovery: Option<Arc<dyn Discovery<Target = T> + Send + Sync>> = None;
        if let Some(target_discovery) = probe_config.target_from() {
            discovery = resolve_discovery::<T>(target_discovery, config).await;
            if discovery.is_none() {
                log::error!(
                    "event=error pipeline={} msg=could_not_resolve_target_from target_from={}",
                    label,
                    target_discovery
                );
                return pipelines;
            }

            if let Some(ref discovery) = discovery {
                targets = discovery.get_targets().await;
            }
        }

        // Override if targets are directly specified
        if let Some(conf_targets) = probe_config.targets() {
            targets = conf_targets.clone();
        }

        if targets.is_empty() {
            log::error!("event=error pipeline={} msg=no_targets_or_target_from_specified", label);
            return pipelines;
        }

        // Resolve exporters
        let resolved_exporters = resolve_exporters(probe_config.forward_to(), exporters);
        log::info!(
            "event=pipeline_created pipeline={} targets={} interval={:?}",
            label,
            targets.len(),
            probe_config.scrape_interval()
        );

        // Create pipeline
        let mut pipeline = Pipeline::new(
            label.to_string(),
            probe_type,
            discovery.clone(),
            probe_init(),
            resolved_exporters,
            probe_config.scrape_interval().to_duration(),
        );
        pipeline.targets = Some(targets);
        pipelines.push(pipeline);

        pipelines
    }

    /// Build all pipelines from the configuration
    /// Returns a Vec of boxed RunnablePipeline trait objects
    pub async fn from_config(
        config: &Configuration,
        exporters: ExportersMap,
    ) -> Vec<Box<dyn RunnablePipeline>> {
        let mut pipelines: Vec<Box<dyn RunnablePipeline>> = Vec::new();

        if let Some(ref probe_wrapper) = config.probe {
            for (label, http_config) in &probe_wrapper.http {
                let http_pipelines = Self::build_pipelines::<
                    configuration::model::target::HttpTarget,
                    HttpProbe,
                    _,
                >(
                    label,
                    http_config,
                    config,
                    &exporters,
                    ProbeType::Http,
                    HttpProbe::init,
                )
                .await;
                for pipeline in http_pipelines {
                    pipelines.push(Box::new(pipeline));
                }
            }

            for (label, icmp_config) in &probe_wrapper.icmp {
                let icmp_pipelines = Self::build_pipelines::<IcmpTarget, IcmpProbe, _>(
                    label,
                    icmp_config,
                    config,
                    &exporters,
                    ProbeType::Icmp,
                    IcmpProbe::init,
                )
                .await;
                for pipeline in icmp_pipelines {
                    pipelines.push(Box::new(pipeline));
                }
            }
        }

        pipelines
    }
}
