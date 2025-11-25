use clap::Parser;
use configuration::{ConfigParser, Discovery, Parse};
use dotenv::dotenv;
use prober::scrap_config::ProbeConfig;
use pyroscope::{
    PyroscopeAgent,
    pyroscope::{PyroscopeAgentReady, PyroscopeAgentRunning},
};
use pyroscope_pprofrs::{PprofConfig, pprof_backend};
use std::{env, io::Error, process::exit, vec};

mod ascii_art;
mod cli;
mod hmr;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    ascii_art::print_ascii_art();
    dotenv().ok();

    let default_config_file_path = match env::var("RUST_ENV").as_deref() {
        Ok("production") => "/etc/zookoo/config.toml",
        _ => "dev/config.toml",
    };

    let args = cli::CliArgs::parse();

    if args.check_config {
        match check_config() {
            Ok(_) => {
                println!("Config is valid");
            }
            Err(err) => {
                println!("INVALID CONFIGURATION");
                println!("{}", err);
            }
        };
    }

    let config_file = match args.config {
        Some(ref f) => f.clone(),
        None => default_config_file_path.to_string(),
    };

    let parser = ConfigParser::new();

    // TODO: REMOVE UNWRAP AND SAFE EXTRACT ERR
    let mut config = parser.parse_from_file(&config_file).unwrap();

    // Enable pyroscope monitoring
    let mut pyroscope_agent: Option<PyroscopeAgent<PyroscopeAgentRunning>> = None;
    if let Some(scrape_config) = config.defaults.self_monitoring.as_ref()
        && scrape_config.enable
    {
        log::warn!(
            "Pyroscope is started and send profiles using '{}'",
            scrape_config.pyroscope_endpoint
        );

        match start_pyrsocope(
            &scrape_config.pyroscope_endpoint,
            &scrape_config.service_name,
        ) {
            Ok(agent) => match agent.start() {
                Ok(started_agent) => {
                    pyroscope_agent = Some(started_agent);
                    log::info!("Pyroscope agent started successfully");
                }
                Err(e) => {
                    log::error!("Failed to start pyroscope agent: {}", e);
                }
            },
            Err(e) => {
                log::error!("Failed to initialize pyroscope agent: {}", e);
            }
        }
    }

    let log_level = args.log_level.unwrap_or(config.defaults.log_level.clone());
    set_log_level(log_level);

    log::info!("Zookoo is launched ! ");
    log::debug!("Config file path={}", config_file);
    log::debug!("{:?}", config);
    log::info!("Starting the probe...");

    // Parse discovery
    if let Err(err) = parser.fetch_discovery(&mut config) {
        log::error!("{}", err);
        exit(1)
    };

    // Run the probe
    prober::run(ProbeConfig::from(config)).await;

    if let Some(agent) = pyroscope_agent {
        let closed_agent = agent.stop().unwrap();
        closed_agent.shutdown();
    }
}

/// Configure the log level and update env logger accordingly
///
/// Caution: Unsafe
/// TODO: maybe find a way to fix it and turn it into safe
fn set_log_level(log_level: String) {
    let default_log_level = String::from("info");

    let log_level_to_apply: String = match log_level.as_str() {
        "error" => String::from("error"),
        "warn" => String::from("warn"),
        "debug" => String::from("debug"),
        "info" => String::from("info"),
        _ => default_log_level,
    };

    unsafe {
        std::env::set_var("RUST_LOG", log_level_to_apply);
    }

    if let Err(err) = env_logger::try_init() {
        println!("Fail to set up env logger. {}", err);
    };
}

fn check_config() -> Result<(), Error> {
    // TODO
    Ok(())
}

fn start_pyrsocope(
    endpoint: &str,
    application_name: &str,
) -> Result<PyroscopeAgent<PyroscopeAgentReady>, Box<dyn std::error::Error>> {
    let pprof_config = PprofConfig::new()
        .report_thread_id()
        .report_thread_name()
        .sample_rate(100);
    let backend_impl = pprof_backend(pprof_config);

    let mut pyroscope = PyroscopeAgent::builder(endpoint, application_name).backend(backend_impl);
    let hostname = hostname::get()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    pyroscope = pyroscope.tags(vec![("host", &hostname)]);

    let agent = pyroscope.build()?;

    Ok(agent)
}
