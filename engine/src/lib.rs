//! # Engine crate
//!
//! This crate is responsible for the scraping process of the different targets defined in the configuration file
//!

use std::collections::HashMap;

use configuration::model::Configuration;
use exporter::types::ExportersMap;
use opentelemetry_sdk::metrics::SdkMeterProvider;

use crate::pipeline::RunnablePipeline;

//mod group_by;
pub(crate) mod factory;
pub(crate) mod pipeline;
pub(crate) mod types;

#[cfg(test)]
mod pipeline_tests;

#[cfg(test)]
mod factory_tests;

pub struct Engine {
    config: Option<Configuration>,
    pipelines: Vec<Box<dyn RunnablePipeline>>,
    exporters: ExportersMap,
    meter_provider: Option<SdkMeterProvider>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            pipelines: Vec::new(),
            config: None,
            exporters: HashMap::new(),
            meter_provider: None,
        }
    }

    /// Load and validate configuration
    #[tracing::instrument(level = "info", skip(self, config))]
    pub fn load_configuration(
        &mut self,
        config: Configuration,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config = Some(config);
        log::info!("event=configuration_loaded");
        Ok(())
    }

    /// Build exporters from configuration
    #[tracing::instrument(level = "info", skip(self))]
    fn build_exporters(&mut self) {
        let config = self.config.as_ref().expect("Configuration must be loaded first");
        let mut exporters: ExportersMap = HashMap::new();
        exporter::build_exporters(config, &mut exporters);

        log::info!("event=exporters_created count={}", exporters.len());
        self.exporters = exporters;
    }

    /// Build pipelines from configuration
    #[tracing::instrument(level = "info", skip(self))]
    async fn build_pipelines(&mut self) {
        let config = self.config.as_ref().expect("Configuration must be loaded first");
        self.pipelines =
            factory::PipelineBuilder::from_config(config, self.exporters.clone()).await;
        log::info!("event=pipelines_created count={}", self.pipelines.len());
    }

    /// Run the engine - initializes providers, builds components, and spawns pipelines
    #[tracing::instrument(level = "info", skip(self))]
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config.is_none() {
            log::error!("event=error msg=no_configuration_loaded");
            return Err("No configuration".into());
        }

        // Phase 2: Build exporters
        self.build_exporters();

        // Phase 3: Build pipelines
        self.build_pipelines().await;

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
