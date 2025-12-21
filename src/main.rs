#![allow(clippy::redundant_async_block)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::unwrap_used)]

use clap::Parser;
use configuration::{ConfigParser, HCL, Parse};
use dotenvy::dotenv;
use pyroscope::{PyroscopeAgent, pyroscope::PyroscopeAgentRunning};

use crate::observability::init_observability;

mod ascii_art;
mod cli;
mod observability;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    ascii_art::print_ascii_art();
    dotenv().ok();

    // Parse CLI args first to get log level
    let args = cli::CliArgs::parse();

    let default_config_file_path = match env::var("RUST_ENV").as_deref() {
        Ok("production") => "/etc/zookoo/config.toml",
        _ => "dev/config.toml",
    };

    let config_file = match args.config {
        Some(ref f) => f.clone(),
        None => default_config_file_path.to_string(),
    };

    let config = match ConfigParser.parse_from_file::<HCL>(&config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            panic!("event=failed_to_parse_config file={} err={}", config_file, e);
        }
    };

    let config_log_level = &config.defaults.log_level;
    // Initialize logging with CLI log level (or default to info)
    let log_level = args.log_level.as_deref().unwrap_or(config_log_level.as_str());

    // If self-monitoring is enabled, init OTLP tracer provider *before* installing the subscriber,
    // so the tracing-opentelemetry layer can export spans.

    let _observability_guard = if let Some(self_monitoring) = config.defaults.self_monitoring.as_ref()
        && self_monitoring.enable
    {
        Some(init_observability(&log_level, self_monitoring.clone()))
    } else {
        None
    };

    // Enable pyroscope monitoring
    let mut pyroscope_agent: Option<PyroscopeAgent<PyroscopeAgentRunning>> = None;
    if let Some(scrape_config) = config.defaults.self_monitoring.as_ref()
        && scrape_config.enable
    {
        log::warn!("event=pyroscope_start endpoint={}", scrape_config.pyroscope_endpoint);

        match observability::start_pyroscope(scrape_config.clone()) {
            Ok(agent) => match agent.start() {
                Ok(started_agent) => {
                    pyroscope_agent = Some(started_agent);
                    log::info!("event=pyroscope_started");
                }
                Err(e) => {
                    log::error!("event=failed_to_start_pyroscope err={}", e);
                }
            },
            Err(e) => {
                log::error!("event=failed_to_init_pyroscope err={}", e);
            }
        }
    }

    log::debug!("event=config_loaded file={}", config_file);

    let mut engine = engine::Engine::new();
    if let Err(e) = engine.load_configuration(config) {
        log::error!("event=failed_to_load_config err={}", e);
        return;
    }

    // Run the engine (blocks until all pipelines complete or error)
    if let Err(e) = engine.run().await {
        log::error!("event=engine_error err={}", e);
    }

    if let Some(agent) = pyroscope_agent {
        let closed_agent = agent.stop().unwrap();
        closed_agent.shutdown();
    }
}
