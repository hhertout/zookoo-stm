#![allow(clippy::redundant_async_block)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::unwrap_used)]

use clap::Parser;
use configuration::{ConfigParser, HCL, Parse, model::defaults::SelfMonitoringConfig};
use dotenvy::dotenv;
use exporter::otlp::meter_provider::init_tracer_provider;
use pyroscope::{
    PyroscopeAgent,
    pyroscope::{PyroscopeAgentReady, PyroscopeAgentRunning},
};
use pyroscope_pprofrs::{PprofConfig, pprof_backend};
use std::{env, io::Error, vec};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, fmt};

use opentelemetry::trace::TracerProvider;

mod ascii_art;
mod cli;

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

    // TODO: create a helper for this
    if args.check_config {
        match check_config() {
            Ok(_) => {
                println!("Configuration is valid !");
            }
            Err(err) => {
                println!("Error, INVALID CONFIGURATION");
                println!("{}", err);
            }
        };
        return;
    }

    let config_file = match args.config {
        Some(ref f) => f.clone(),
        None => default_config_file_path.to_string(),
    };

    let config = match ConfigParser.parse_from_file::<HCL>(&config_file) {
        Ok(cfg) => cfg,
        Err(e) => {
            panic!("event=error msg=failed_to_parse_config file={} err={}", config_file, e);
        }
    };

    let config_log_level = &config.defaults.log_level;
    // Initialize logging with CLI log level (or default to info)
    let log_level = args.log_level.as_deref().unwrap_or(config_log_level.as_str());

    // If self-monitoring is enabled, init OTLP tracer provider *before* installing the subscriber,
    // so the tracing-opentelemetry layer can export spans.
    let mut tracer_provider = None;
    if let Some(self_monitoring) = config.defaults.self_monitoring.as_ref()
        && self_monitoring.enable
    {
        tracer_provider = Some(init_tracer_provider(self_monitoring.clone()));
    }

    init_logging(log_level, tracer_provider.as_ref());

    // Enable pyroscope monitoring
    let mut pyroscope_agent: Option<PyroscopeAgent<PyroscopeAgentRunning>> = None;
    if let Some(scrape_config) = config.defaults.self_monitoring.as_ref()
        && scrape_config.enable
    {
        tracing::warn!("event=pyroscope_start endpoint={}", scrape_config.pyroscope_endpoint);

        match start_pyrsocope(scrape_config.clone()) {
            Ok(agent) => match agent.start() {
                Ok(started_agent) => {
                    pyroscope_agent = Some(started_agent);
                    tracing::info!("event=pyroscope_started");
                }
                Err(e) => {
                    tracing::error!("event=error msg=failed_to_start_pyroscope err={}", e);
                }
            },
            Err(e) => {
                tracing::error!("event=error msg=failed_to_init_pyroscope err={}", e);
            }
        }
    }

    tracing::debug!("event=config_loaded file={}", config_file);

    let mut engine = engine::Engine::new();
    if let Err(e) = engine.load_configuration(config) {
        tracing::error!("event=error msg=failed_to_load_config err={}", e);
        return;
    }

    // Run the engine (blocks until all pipelines complete or error)
    if let Err(e) = engine.run().await {
        tracing::error!("event=error msg=engine_error err={}", e);
    }

    if let Some(agent) = pyroscope_agent {
        let closed_agent = agent.stop().unwrap();
        closed_agent.shutdown();
    }
}

/// Configure the log level and initialize tracing subscriber
///
/// This sets up tracing-subscriber which captures both:
/// - Regular log crate messages (via tracing-log)
/// - Internal OpenTelemetry logs (via internal-logs feature)
///
/// The OTEL log level is derived from RUST_LOG:
/// - debug/trace -> opentelemetry_otlp=debug (shows export errors)
/// - info/warn/error -> opentelemetry_otlp=warn (quieter)
fn init_logging(
    log_level: &str,
    tracer_provider: Option<&opentelemetry_sdk::trace::SdkTracerProvider>,
) {
    let log_level_to_apply = match log_level {
        "error" | "warn" | "debug" | "info" | "trace" => log_level,
        _ => "info",
    };

    // Derive OTEL log level from app log level
    // If debug/trace, show OTEL debug logs (including export errors)
    // Otherwise, only show warnings
    let otel_level = match log_level_to_apply {
        "debug" | "trace" => "debug",
        _ => "warn",
    };

    // Build filter: app logs + OTEL logs at derived level + quiet noisy deps
    let filter = format!(
        "{},opentelemetry={},opentelemetry_sdk={},opentelemetry_otlp={},h2=warn,hyper=warn,tower=warn,tonic=warn",
        log_level_to_apply, otel_level, otel_level, otel_level
    );

    let registry = tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&filter)))
        .with(fmt::layer().with_target(true));

    if let Some(provider) = tracer_provider {
        // Bridge tracing spans/events to OpenTelemetry.
        // Use the concrete SDK tracer so it satisfies PreSampledTracer.
        let tracer = provider.tracer("zookoo");
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        registry.with(otel_layer).init();
    } else {
        registry.init();
    }
}

fn check_config() -> Result<(), Error> {
    // TODO
    Ok(())
}

fn start_pyrsocope(
    config: SelfMonitoringConfig,
) -> Result<PyroscopeAgent<PyroscopeAgentReady>, Box<dyn std::error::Error>> {
    let pprof_config = PprofConfig::new().report_thread_id().report_thread_name().sample_rate(100);
    let backend_impl = pprof_backend(pprof_config);

    let mut pyroscope = PyroscopeAgent::builder(config.pyroscope_endpoint, config.service_name)
        .backend(backend_impl);
    let hostname = hostname::get().unwrap_or_default().to_string_lossy().to_string();

    if let Some(basic_auth) = config.basic_auth {
        pyroscope = pyroscope.basic_auth(basic_auth.username, basic_auth.password);
    }

    if let Some(bearer) = config.bearer {
        pyroscope = pyroscope.auth_token(bearer);
    }

    pyroscope = pyroscope.tags(vec![("host", &hostname)]);

    let agent = pyroscope.build()?;

    Ok(agent)
}
