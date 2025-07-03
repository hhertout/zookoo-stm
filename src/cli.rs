#[derive(clap::Parser, Debug)]
#[command(author, version, about, name = "Rustbox")]
pub struct CliArgs {
    /// <CONFIG> = Configuration file location
    #[arg(long)]
    pub config: Option<String>,

    /// Configure the log level - error, warn, info, debug
    #[arg(long)]
    pub log_level: Option<String>,

    /// Execute a precheck and ensure the configuration file is correctly formated
    #[arg(long)]
    pub check_config: bool,
}
