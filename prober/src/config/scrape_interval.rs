use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub enum ScrapeInterval {
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

impl ScrapeInterval {
    pub fn to_duration(&self) -> Duration {
        match self {
            ScrapeInterval::S5 => Duration::from_secs(5),
            ScrapeInterval::S10 => Duration::from_secs(10),
            ScrapeInterval::S30 => Duration::from_secs(30),
            ScrapeInterval::M1 => Duration::from_secs(60),
            ScrapeInterval::M5 => Duration::from_secs(5 * 60),
            ScrapeInterval::M10 => Duration::from_secs(10 * 60),
            ScrapeInterval::M30 => Duration::from_secs(30 * 60),
            ScrapeInterval::H1 => Duration::from_secs(60 * 60),
            ScrapeInterval::H12 => Duration::from_secs(12 * 60 * 60),
            ScrapeInterval::D1 => Duration::from_secs(24 * 60 * 60),
            ScrapeInterval::D7 => Duration::from_secs(7 * 24 * 60 * 60),
            ScrapeInterval::D30 => Duration::from_secs(30 * 24 * 60 * 60),
        }
    }
}

impl ToString for ScrapeInterval {
    fn to_string(&self) -> String {
        match self {
            ScrapeInterval::S5 => "5s".to_owned(),
            ScrapeInterval::S10 => "10s".to_owned(),
            ScrapeInterval::S30 => "30s".to_owned(),
            ScrapeInterval::M1 => "1m".to_owned(),
            ScrapeInterval::M5 => "5m".to_owned(),
            ScrapeInterval::M10 => "10m".to_owned(),
            ScrapeInterval::M30 => "30m".to_owned(),
            ScrapeInterval::H1 => "1h".to_owned(),
            ScrapeInterval::H12 => "12h".to_owned(),
            ScrapeInterval::D1 => "1d".to_owned(),
            ScrapeInterval::D7 => "7d".to_owned(),
            ScrapeInterval::D30 => "30d".to_owned(),
        }
    }
}

impl From<configuration::model::scrape_interval::ScrapeInterval> for ScrapeInterval {
    fn from(value: configuration::model::scrape_interval::ScrapeInterval) -> Self {
        match value {
            configuration::model::scrape_interval::ScrapeInterval::S5 => ScrapeInterval::S5,
            configuration::model::scrape_interval::ScrapeInterval::S10 => ScrapeInterval::S10,
            configuration::model::scrape_interval::ScrapeInterval::S30 => ScrapeInterval::S30,
            configuration::model::scrape_interval::ScrapeInterval::M1 => ScrapeInterval::M1,
            configuration::model::scrape_interval::ScrapeInterval::M5 => ScrapeInterval::M5,
            configuration::model::scrape_interval::ScrapeInterval::M10 => ScrapeInterval::M10,
            configuration::model::scrape_interval::ScrapeInterval::M30 => ScrapeInterval::M30,
            configuration::model::scrape_interval::ScrapeInterval::H1 => ScrapeInterval::H1,
            configuration::model::scrape_interval::ScrapeInterval::H12 => ScrapeInterval::H12,
            configuration::model::scrape_interval::ScrapeInterval::D1 => ScrapeInterval::D1,
            configuration::model::scrape_interval::ScrapeInterval::D7 => ScrapeInterval::D7,
            configuration::model::scrape_interval::ScrapeInterval::D30 => ScrapeInterval::D30,
        }
    }
}
