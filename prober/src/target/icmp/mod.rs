use tokio::sync::mpsc;

use crate::target::Scraping;
use crate::{scrap_config::ProbeConfig, target::icmp::scrape::IcmpScrapper};
use std::sync::Arc;

pub(crate) mod ping;
pub(crate) mod scrape;

pub async fn icmp_scrape_with_shutdown(config: ProbeConfig, mut shutdown_rx: mpsc::Receiver<()>) {
    let icmp_group_by = config.icmp_group_by_interval();
    let mut tasks = Vec::new();

    for (interval, items) in icmp_group_by {
        if !items.is_empty() {
            let scrapper = Arc::new(IcmpScrapper { targets: items });
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
