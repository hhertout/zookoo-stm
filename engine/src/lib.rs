//! # Engine crate
//!
//! This crate is responsible for the scraping process of the different targets defined in the configuration file
//!

use std::{collections::HashMap, sync::Arc};

use configuration::model::Configuration;
use exporter::{Exporter, otel::otel_exporter::OtelExporter};
use opentelemetry_sdk::{metrics::SdkMeterProvider, trace::SdkTracerProvider};
use probe::observability::{init_meter_provider, init_tracer_provider};

use crate::pipeline::{PipelineBuilder, RunnablePipeline};

pub(crate) mod defaults_labels;
mod group_by;
pub(crate) mod pipeline;
pub(crate) mod types;

/// Type alias for labeled exporters map
pub type ExportersMap = HashMap<String, Arc<dyn Exporter + Send + Sync>>;

pub struct Engine {
    config: Option<Configuration>,
    pipelines: Vec<Box<dyn RunnablePipeline>>,
    exporters: ExportersMap,
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            pipelines: Vec::new(),
            config: None,
            exporters: HashMap::new(),
            tracer_provider: None,
            meter_provider: None,
        }
    }

    /// Load and validate configuration
    pub fn load_configuration(
        &mut self,
        config: Configuration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config = Some(config);
        log::info!("event=configuration_loaded");
        Ok(())
    }

    /// Initialize observability (tracing and metrics providers)
    fn init_observability(&mut self) {
        let config = self.config.as_ref().expect("Configuration must be loaded first");

        // Initialize meter provider from first OTEL exporter
        if let Some(ref exporter_wrapper) = config.exporter
            && let Some((label, otel_config)) = exporter_wrapper.otel.iter().next()
        {
            log::info!("event=init_meter_provider exporter={} endpoint={}", label, otel_config.url);
            self.meter_provider = Some(init_meter_provider(
                otel_config.url.clone(),
                "zookoo".to_string(),
                "production".to_string(),
                config.defaults.probe_zone.clone(),
            ));
        }

        // Initialize tracer provider if self-monitoring is enabled
        if let Some(ref self_monitoring) = config.defaults.self_monitoring
            && self_monitoring.enable
        {
            log::info!("event=init_tracer_provider");
            self.tracer_provider = Some(init_tracer_provider(
                self_monitoring.otel_endpoint.clone(),
                self_monitoring.service_name.clone(),
                self_monitoring.env.clone(),
                config.defaults.probe_zone.clone(),
            ));
        }
    }

    /// Build exporters from configuration
    fn build_exporters(&mut self) {
        let config = self.config.as_ref().expect("Configuration must be loaded first");
        let mut exporters: ExportersMap = HashMap::new();

        if let Some(ref exporter_wrapper) = config.exporter {
            // Build OTEL exporters
            for (label, otel_config) in &exporter_wrapper.otel {
                let key = format!("exporter.otel.{}", label);
                log::info!(
                    "event=create_exporter type=otel key={} endpoint={}",
                    key,
                    otel_config.url
                );

                let mut override_labels: HashMap<String, String> = HashMap::new();
                override_labels.insert("exporter".to_string(), label.clone());
                let labels =
                    defaults_labels::set_defaults_labels(&config.defaults, override_labels);

                let exporter = OtelExporter::new(labels);
                exporters.insert(key, Arc::new(exporter));
            }

            // TODO: Add other exporter types
            for label in exporter_wrapper.prometheus_remote_write.keys() {
                log::warn!(
                    "event=exporter_not_implemented type=prometheus_remote_write label={}",
                    label
                );
            }
            for label in exporter_wrapper.timescale.keys() {
                log::warn!("event=exporter_not_implemented type=timescale label={}", label);
            }
            for label in exporter_wrapper.kafka.keys() {
                log::warn!("event=exporter_not_implemented type=kafka label={}", label);
            }
        }

        log::info!("event=exporters_created count={}", exporters.len());
        self.exporters = exporters;
    }

    /// Build pipelines from configuration
    fn build_pipelines(&mut self) {
        let config = self.config.as_ref().expect("Configuration must be loaded first");
        self.pipelines = PipelineBuilder::from_config(config, self.exporters.clone());
        log::info!("event=pipelines_created count={}", self.pipelines.len());
    }

    /// Run the engine - initializes providers, builds components, and spawns pipelines
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_none() {
            log::error!("event=error msg=no_configuration_loaded");
            return Err("No configuration".into());
        }

        // Phase 1: Initialize observability (meter & tracer providers)
        self.init_observability();

        // Phase 2: Build exporters
        self.build_exporters();

        // Phase 3: Build pipelines
        self.build_pipelines();

        if self.pipelines.is_empty() {
            log::warn!("event=no_pipelines");
            return Ok(());
        }

        // Phase 4: Spawn and run pipelines
        log::info!("event=starting_pipelines count={}", self.pipelines.len());

        let pipelines = std::mem::take(&mut self.pipelines);
        let mut handles = Vec::new();

        for mut pipeline in pipelines {
            let label = pipeline.label().to_string();
            let handle = tokio::spawn(async move {
                log::info!("event=pipeline_started label={}", label);
                pipeline.run().await;
            });
            handles.push(handle);
        }

        for handle in handles {
            let _ = handle.await;
        }

        Ok(())
    }

    /// Graceful shutdown
    pub fn shutdown(&mut self) {
        if let Some(provider) = self.meter_provider.take() {
            log::info!("event=shutdown component=meter_provider");
            let _ = provider.shutdown();
        }
        if let Some(provider) = self.tracer_provider.take() {
            log::info!("event=shutdown component=tracer_provider");
            let _ = provider.shutdown();
        }
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        self.shutdown();
    }
}
