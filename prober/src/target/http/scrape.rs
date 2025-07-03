use crate::target::Scraping;
use std::{sync::Arc, time::Duration};

use tokio::{task::JoinHandle, time::interval};

use crate::{config::target::HttpTarget, scrap_config::ProbeConfig, target::http::HttpScrapper};

pub async fn http_scrape(config: ProbeConfig) -> Vec<JoinHandle<()>> {
    let json_group_by = config.json_http_group_by_interval();
    let http_group_by = config.http_group_by_interval();

    let merged = json_group_by.merge(http_group_by);

    let mut handles = Vec::new();
    for (interval, items) in merged {
        if !items.is_empty() {
            let items = items.to_vec();
            let duration = Duration::from(interval.to_duration());

            let handle = tokio::spawn(async move {
                launch_http_scrape(duration, items).await;
            });

            handles.push(handle);
        }
    }

    handles
}

async fn launch_http_scrape(interval_duration: Duration, targets: Vec<HttpTarget>) {
    let mut ticker = interval(interval_duration);
    let scrapper = Arc::new(HttpScrapper { targets });
    loop {
        ticker.tick().await;

        let scrapper = Arc::clone(&scrapper);
        tokio::spawn(async move {
            if let Err(e) = scrapper.scrape().await {
                log::error!("Scrape failed: {:?}", e);
            }
            Ok::<(), ()>(())
        });
    }
}
