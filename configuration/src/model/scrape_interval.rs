use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScrapeInterval {
    #[serde(rename = "5s")]
    S5,
    #[serde(rename = "10s")]
    S10,
    #[serde(rename = "15s")]
    S15,
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
            ScrapeInterval::S15 => Duration::from_secs(15),
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

/// Trait for targets that have a scrape interval
pub trait HasScrapeInterval {
    fn scrape_interval(&self) -> ScrapeInterval;
}
