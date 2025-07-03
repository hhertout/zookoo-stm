use clap::Parser;
use configuration::{ConfigParser, Parse};
use dotenv::dotenv;
use prober::scrap_config::ProbeConfig;
use std::{env, io::Error};

mod cli;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    dotenv().ok();

    let default_config_file_path = match env::var("RUST_ENV").as_deref() {
        Ok("production") => "/etc/rustbox/config.toml",
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
    let config = parser.parse_from_file(&config_file).unwrap();

    let log_level = args
        .log_level
        .or(Some(config.defaults.log_level.clone()))
        .unwrap();
    set_log_level(log_level);

    log::info!("Rustbox is launched ! ");
    log::debug!("Config file path={}", config_file);
    log::debug!("{:?}", config);
    log::info!("Starting the probe...");

    prober::start_probe(ProbeConfig::from(config)).await;
}

/// Configure the log level and update env logger accordingly
fn set_log_level(log_level: String) {
    let default_log_level = String::from("info");

    let log_level_to_apply: String;
    match log_level.as_str() {
        "error" => log_level_to_apply = String::from("error"),
        "warn" => log_level_to_apply = String::from("warn"),
        "debug" => log_level_to_apply = String::from("debug"),
        "info" => log_level_to_apply = String::from("info"),
        _ => log_level_to_apply = default_log_level,
    }

    unsafe {
        env::set_var("RUST_LOG", log_level_to_apply);
    }

    if let Err(err) = env_logger::try_init() {
        println!("Fail to set up env logger. {}", err);
    };
}

fn check_config() -> Result<(), Error> {
    Ok(())
}
