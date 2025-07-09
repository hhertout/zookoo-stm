use clap::Parser;
use configuration::{ConfigParser, Parse};
use dotenv::dotenv;
use prober::scrap_config::ProbeConfig;
use pyroscope::{
    PyroscopeAgent,
    pyroscope::{PyroscopeAgentReady, PyroscopeAgentRunning},
};
use pyroscope_pprofrs::{PprofConfig, pprof_backend};
use std::{env, io::Error, vec};

mod cli;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    print_ascii_art();
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
    let config = parser.parse_from_file(&config_file).unwrap();

    // Enable pyroscope monitoring
    let mut pyroscope_agent: Option<PyroscopeAgent<PyroscopeAgentRunning>> = None;
    if config.defaults.self_monitoring.enable {
        log::warn!(
            "Pyroscope is started and send profiles using '{}'",
            config.defaults.self_monitoring.pyroscope_endpoint
        );

        if let Ok(agent) = start_pyrsocope(
            &config.defaults.self_monitoring.pyroscope_endpoint,
            &config.defaults.self_monitoring.service_name,
        ) {
            pyroscope_agent = Some(agent.start().unwrap());
        } else {
            log::error!("fail to start pyroscope agent")
        };
    }

    let log_level = args.log_level.unwrap_or(config.defaults.log_level.clone());
    set_log_level(log_level);

    log::info!("Zookoo is launched ! ");
    log::debug!("Config file path={}", config_file);
    log::debug!("{:?}", config);
    log::info!("Starting the probe...");

    prober::run(ProbeConfig::from(config)).await;

    if let Some(agent) = pyroscope_agent {
        let closed_agent = agent.stop().unwrap();
        closed_agent.shutdown();
    }
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
        std::env::set_var("RUST_LOG", log_level_to_apply);
    }

    if let Err(err) = env_logger::try_init() {
        println!("Fail to set up env logger. {}", err);
    };
}

fn check_config() -> Result<(), Error> {
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

fn print_ascii_art() {
    println!(
        r#"
                _
            ,.-" "-.,
           /   ===   \
          /  =======  \
       __|  (o)   (0)  |__      
      / _|    .---.    |_ \         
     | /.----/ O O \----.\ |       
      \/     |     |     \/        
      |                   |            
      |                   |           
      |                   |          
      _\   -.,_____,.-   /_         
  ,.-"  "-.,_________,.-"  "-.,
 /          |       |          \  
|           l.     .l           | 
|            |     |            |
l.           |     |           .l             
 |           l.   .l           | \,     
 l.           |   |           .l   \,    
  |           |   |           |      \,  
  l.          |   |          .l        |
   |          |   |          |         |
   |          |---|          |         |
   |          |   |          |         |
   /"-.,__,.-"\   /"-.,__,.-"\"-.,_,.-"\
  |            \ /            |         |
  |             |             |         |
   \__|__|__|__/ \__|__|__|__/ \_|__|__/

    ______            _    _____  _____ 
   |___  /           | |  |  _  ||  _  |
      / /  ___   ___ | | _| | | || | | |
     / /  / _ \ / _ \| |/ / | | || | | |
   ./ /__| (_) | (_) |   <\ \_/ /\ \_/ /
   \_____/\___/ \___/|_|\_\\___/  \___/ 
                                        
                                     
"#
    );
}
