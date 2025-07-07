//!
//! ## Target module
//!
//! Made to define the behavior of the scraping process depending on the target type
//!
//! ### Target type supported
//! - ICMP
//! - HTTP
//! - HTTPS
//!
use std::fmt::Display;
use std::sync::Arc;

use opentelemetry::Context;
use tokio::sync::mpsc;

use crate::group_by_interval::GroupByInterval;

pub mod http;
pub mod icmp;

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    HTTP,
    HTTPS,
    IPV4,
}

impl ToString for TargetType {
    fn to_string(&self) -> String {
        match self {
            TargetType::HTTP => String::from("HTTP"),
            TargetType::HTTPS => String::from("HTTPS"),
            TargetType::IPV4 => String::from("ipv4"),
        }
    }
}

#[derive(Debug)]
pub enum ScrapeError {
    TypeError(String),
    InvalidInput(String),
    LookupFailed,
    NetworkError(String),
}

impl Display for ScrapeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TypeError(s) => write!(f, "type error = {}", s),
            Self::InvalidInput(s) => write!(f, "invalid input error = {}", s),
            Self::LookupFailed => write!(f, "lookup failed"),
            Self::NetworkError(s) => write!(f, "network error = {}", s),
        }
    }
}

impl std::error::Error for ScrapeError {}

pub trait Scraping<T> {
    fn new(targets: Vec<T>) -> Self;
    fn scrape(&self) -> impl Future<Output = Result<(), ScrapeError>> + Send;
    fn send_request(
        &self,
        target: &T,
        cx: Context,
    ) -> impl Future<Output = Result<(), ScrapeError>> + Send;
}

pub async fn scrape_with_shutdown<T, G>(
    intervals_scraping: GroupByInterval<G>,
    mut shutdown_rx: mpsc::Receiver<()>,
) where
    T: Scraping<G> + Sync + std::marker::Send + 'static,
{
    let mut tasks = Vec::new();

    for (interval, items) in intervals_scraping {
        if !items.is_empty() {
            let scrapper = Arc::new(T::new(items));
            let (shutdown_tx, mut task_shutdown_rx) = mpsc::channel::<()>(1);

            let task = tokio::spawn(async move {
                let mut ticker = tokio::time::interval(interval.to_duration());
                loop {
                    tokio::select! {
                        _ = ticker.tick() => {
                            let scrapper = Arc::clone(&scrapper);
                            tokio::spawn(async move {
                                if let Err(e) = scrapper.scrape().await {
                                    log::error!("scrape failed: {:?}", e);
                                }
                            });
                        }
                        _ = task_shutdown_rx.recv() => {
                            log::debug!("Shutting down scraping task for {:?}.", interval.to_duration());
                            break;
                        }
                    }
                }
                log::debug!(
                    "Scraping task for {:?} exited loop.",
                    interval.to_duration()
                );
            });

            tasks.push((task, shutdown_tx));
        }
    }

    log::info!("All icmp scraping tasks spawned. Waiting for main shutdown signal.");
    shutdown_rx.recv().await;
    log::info!("Main shutdown signal received. Sending shutdown to tasks...");
    for (task, shutdown_tx) in tasks {
        let _ = shutdown_tx.send(()).await;
        let _ = task.await;
    }
    log::info!("All icmp scraping tasks shut down.");
}
