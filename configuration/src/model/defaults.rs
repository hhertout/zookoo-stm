use serde::Deserialize;

fn default_log_level() -> String {
    return String::from("info");
}

#[derive(Debug, Clone, Deserialize)]
pub struct Defaults {
    #[serde(default = "default_log_level")]
    pub log_level: String,
}
