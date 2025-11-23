//! Scraper trait definition
//!
//! This module defines the core trait that all scraper implementations must follow

use std::fmt::Display;
use std::sync::Arc;
use opentelemetry::Context;
use tokio::sync::mpsc;
use crate::utils::group_by_interval::GroupByInterval;

/// Error types for scraping operations
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

/// Trait to define the scraping behavior for different probe types
///
/// This trait must be implemented by all probe types (HTTP, ICMP, etc.)
/// to define how they scrape their targets and handle the results.
///
/// # Type Parameters
/// * `T` - The target type (e.g., HttpTarget, IcmpTarget)
pub trait Scraping<T>: Send + Sync + 'static {
    /// Create a new scraper instance with the given targets
    fn new(targets: Vec<T>) -> Self;
    
    /// Execute the scraping operation for all targets
    fn scrape(&self) -> impl Future<Output = Result<(), ScrapeError>> + Send;
    
    /// Send a request to a specific target
    fn send_request(
        &self,
        target: &T,
        cx: Context,
    ) -> impl Future<Output = Result<(), ScrapeError>> + Send;
}

/// Main method to scrape targets with graceful shutdown support
///
/// This function manages the lifecycle of scraping tasks, including:
/// - Creating interval-based scraping loops
/// - Managing shutdown signals
/// - Ensuring all tasks complete gracefully
///
/// # Type Parameters
/// * `T` - The scraper type implementing the Scraping trait
/// * `G` - The target configuration type
pub async fn scrape_with_shutdown<T, G>(
    intervals_scraping: GroupByInterval<G>,
    mut shutdown_rx: mpsc::Receiver<()>,
) where
    T: Scraping<G>,
{
    let mut tasks = Vec::new();

    for (interval, items) in intervals_scraping {
        if !items.is_empty() {
            let scrapper = Arc::new(T::new(items));
            let (shutdown_tx, mut task_shutdown_rx) = mpsc::channel::<()>(1);

            let task = tokio::spawn(async move {
                let mut ticker = tokio::time::interval(interval.to_duration());
                let mut scrape_tasks = Vec::new();
                
                loop {
                    tokio::select! {
                        _ = ticker.tick() => {
                            let scrapper = Arc::clone(&scrapper);
                            let handle = tokio::spawn(async move {
                                if let Err(e) = scrapper.scrape().await {
                                    log::error!("scrape failed: {:?}", e);
                                }
                            });
                            scrape_tasks.push(handle);
                        }
                        _ = task_shutdown_rx.recv() => {
                            log::debug!("Shutting down scraping task for {:?}.", interval.to_duration());
                            break;
                        }
                    }
                }
                
                // Wait for all pending scrape tasks to complete
                for handle in scrape_tasks {
                    let _ = handle.await;
                }
                log::debug!(
                    "Scraping task for {:?} exited loop.",
                    interval.to_duration()
                );
            });

            tasks.push((task, shutdown_tx));
        }
    }

    log::info!("All scraping tasks spawned. Waiting for main shutdown signal.");
    shutdown_rx.recv().await;
    log::info!("Main shutdown signal received. Sending shutdown to tasks...");
    
    for (task, shutdown_tx) in tasks {
        let _ = shutdown_tx.send(()).await;
        let _ = task.await;
    }
    
    log::info!("All scraping tasks shut down.");
}
