use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum ScrapInterval {
    #[serde(rename = "5s")]
    S5,
    #[serde(rename = "10s")]
    S10,
    #[serde(rename = "30s")]
    S30,
    #[serde(rename = "1m")]
    M1,
    #[serde(rename = "5m")]
    M5,
    #[serde(rename = "10m")]
    M10,
    #[serde(rename = "30m")]
    M30,
    #[serde(rename = "1h")]
    H1,
    #[serde(rename = "12h")]
    H12,
    #[serde(rename = "1d")]
    D1,
    #[serde(rename = "7d")]
    D7,
    #[serde(rename = "30d")]
    D30,
}

impl ScrapInterval {
    pub fn to_duration(&self) -> Duration {
        match self {
            ScrapInterval::S5 => Duration::from_secs(5),
            ScrapInterval::S10 => Duration::from_secs(10),
            ScrapInterval::S30 => Duration::from_secs(30),
            ScrapInterval::M1 => Duration::from_secs(60),
            ScrapInterval::M5 => Duration::from_secs(5 * 60),
            ScrapInterval::M10 => Duration::from_secs(10 * 60),
            ScrapInterval::M30 => Duration::from_secs(30 * 60),
            ScrapInterval::H1 => Duration::from_secs(60 * 60),
            ScrapInterval::H12 => Duration::from_secs(12 * 60 * 60),
            ScrapInterval::D1 => Duration::from_secs(24 * 60 * 60),
            ScrapInterval::D7 => Duration::from_secs(7 * 24 * 60 * 60),
            ScrapInterval::D30 => Duration::from_secs(30 * 24 * 60 * 60),
        }
    }
}

impl ToString for ScrapInterval {
    fn to_string(&self) -> String {
        match self {
            ScrapInterval::S5 => "5s".to_owned(),
            ScrapInterval::S10 => "10s".to_owned(),
            ScrapInterval::S30 => "30s".to_owned(),
            ScrapInterval::M1 => "1m".to_owned(),
            ScrapInterval::M5 => "5m".to_owned(),
            ScrapInterval::M10 => "10m".to_owned(),
            ScrapInterval::M30 => "30m".to_owned(),
            ScrapInterval::H1 => "1h".to_owned(),
            ScrapInterval::H12 => "12h".to_owned(),
            ScrapInterval::D1 => "1d".to_owned(),
            ScrapInterval::D7 => "7d".to_owned(),
            ScrapInterval::D30 => "30d".to_owned(),
        }
    }
}

impl From<configuration::model::scrap_interval::ScrapInterval> for ScrapInterval {
    fn from(value: configuration::model::scrap_interval::ScrapInterval) -> Self {
        match value {
            configuration::model::scrap_interval::ScrapInterval::S5 => ScrapInterval::S5,
            configuration::model::scrap_interval::ScrapInterval::S10 => ScrapInterval::S10,
            configuration::model::scrap_interval::ScrapInterval::S30 => ScrapInterval::S30,
            configuration::model::scrap_interval::ScrapInterval::M1 => ScrapInterval::M1,
            configuration::model::scrap_interval::ScrapInterval::M5 => ScrapInterval::M5,
            configuration::model::scrap_interval::ScrapInterval::M10 => ScrapInterval::M10,
            configuration::model::scrap_interval::ScrapInterval::M30 => ScrapInterval::M30,
            configuration::model::scrap_interval::ScrapInterval::H1 => ScrapInterval::H1,
            configuration::model::scrap_interval::ScrapInterval::H12 => ScrapInterval::H12,
            configuration::model::scrap_interval::ScrapInterval::D1 => ScrapInterval::D1,
            configuration::model::scrap_interval::ScrapInterval::D7 => ScrapInterval::D7,
            configuration::model::scrap_interval::ScrapInterval::D30 => ScrapInterval::D30,
        }
    }
}
