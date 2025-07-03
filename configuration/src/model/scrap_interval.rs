use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
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
