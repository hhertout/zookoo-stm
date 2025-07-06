use crate::{
    scrap_config::ProbeConfig,
    target::{Scraping, http::scrape::HttpScrapper},
};
use std::sync::Arc;
use tokio::sync::mpsc;

pub(crate) mod dns;
pub(crate) mod request;
pub mod scrape;
pub(crate) mod tls;

pub async fn http_scrape_with_shutdown(config: ProbeConfig, mut shutdown_rx: mpsc::Receiver<()>) {
    let json_group_by = config.json_http_group_by_interval();
    let http_group_by = config.http_group_by_interval();
    let mut tasks = Vec::new();

    let merged = json_group_by.merge(http_group_by);

    for (interval, items) in merged {
        if !items.is_empty() {
            let scrapper = Arc::new(HttpScrapper { targets: items });
            let (shutdown_tx, mut task_shutdown_rx) = mpsc::channel::<()>(1);

            let task = tokio::spawn(async move {
                let mut ticker = tokio::time::interval(interval.to_duration());

                loop {
                    tokio::select! {
                        _ = ticker.tick() => {
                            let scrapper = Arc::clone(&scrapper);
                            tokio::spawn(async move { scrapper.scrape().await });
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

    log::info!("All http scraping tasks spawned. Waiting for main shutdown signal.");
    shutdown_rx.recv().await;
    log::info!("Main shutdown signal received. Sending shutdown to tasks...");
    for (task, shutdown_tx) in tasks {
        let _ = shutdown_tx.send(()).await;
        let _ = task.await;
    }
    log::info!("All http scraping tasks shut down.");
}
