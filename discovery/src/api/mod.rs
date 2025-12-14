use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use async_trait::async_trait;
use configuration::model::{RefreshInterval, discovery::DiscoveryApi};
use reqwest::header;
use serde::de::DeserializeOwned;
use tokio::sync::RwLock;
use tokio::sync::watch;

use crate::Discovery;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct ApiDiscovery<T: Clone + std::fmt::Debug + Send + Sync + 'static> {
    url: String,
    headers: Option<HashMap<String, String>>,
    basic_auth: Option<String>,
    bearer: Option<String>,
    refresh_interval: RefreshInterval,
    targets: Arc<RwLock<Vec<T>>>,
    update_started: Arc<AtomicBool>,
    version: Arc<AtomicU64>,
    update_tx: watch::Sender<u64>,
    update_rx: watch::Receiver<u64>,
}

impl<T> ApiDiscovery<T>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    pub fn new(conf: DiscoveryApi) -> Self {
        let (update_tx, update_rx) = watch::channel(0u64);
        // TODO: Use conf object
        Self {
            url: conf.url,
            headers: conf.headers,
            basic_auth: conf.basic_auth,
            bearer: conf.bearer,
            refresh_interval: conf.refresh_interval,
            targets: Arc::new(RwLock::new(Vec::new())),
            update_started: Arc::new(AtomicBool::new(false)),
            version: Arc::new(AtomicU64::new(0)),
            update_tx,
            update_rx,
        }
    }

    /// Fetch and update targets from the API
    /// If fetching fails (url_not_reachable), keep existing targets
    /// If parsing fails, log error and keep existing targets
    async fn get_targets_from_api(
        &self,
    ) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let headers_map = self.to_headers_map();
        let data =
            client.get(&self.url).headers(headers_map).send().await?.json::<Vec<T>>().await?;

        Ok(data)
    }

    fn to_headers_map(&self) -> reqwest::header::HeaderMap {
        let mut headers_map = reqwest::header::HeaderMap::new();

        if let Some(headers) = &self.headers {
            for (key, value) in headers.iter() {
                let name = match reqwest::header::HeaderName::from_bytes(key.as_bytes()) {
                    Ok(name) => name,
                    Err(e) => {
                        log::warn!(
                            "event=warn msg=API_DISCOVERY_INVALID_HEADER_NAME remediation=skipping name={} err={}",
                            key,
                            e
                        );
                        continue;
                    }
                };

                let value = match reqwest::header::HeaderValue::from_str(value) {
                    Ok(value) => value,
                    Err(e) => {
                        log::warn!(
                            "event=warn msg=API_DISCOVERY_INVALID_HEADER_VALUE remediation=skipping name={} err={}",
                            key,
                            e
                        );
                        continue;
                    }
                };

                headers_map.insert(name, value);
            }
        }

        if let Some(basic_auth) = &self.basic_auth {
            match reqwest::header::HeaderValue::from_str(&format!("Basic {}", basic_auth)) {
                Ok(auth_value) => {
                    headers_map.insert(header::AUTHORIZATION, auth_value);
                }
                Err(e) => {
                    log::warn!(
                        "event=warn msg=API_DISCOVERY_INVALID_AUTHORIZATION remediation=skipping err={}",
                        e
                    );
                }
            }
        }

        if let Some(bearer) = &self.bearer {
            match reqwest::header::HeaderValue::from_str(&format!("Bearer {}", bearer)) {
                Ok(bearer_value) => {
                    headers_map.insert(header::AUTHORIZATION, bearer_value);
                }
                Err(e) => {
                    log::warn!(
                        "event=warn msg=API_DISCOVERY_INVALID_BEARER remediation=skipping err={}",
                        e
                    );
                }
            }
        }

        headers_map
    }
}

#[async_trait]
impl<T> Discovery for ApiDiscovery<T>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    type Target = T;

    async fn discover(&self) {
        // Keep last known state if the API is unreachable or returns invalid data.
        match self.get_targets_from_api().await {
            Ok(data) => {
                log::info!("event=info msg=api_discovery_success url={}", self.url);
                self.targets.write().await.clone_from(&data);
                self.version.fetch_add(1, Ordering::Relaxed);
                let _ = self.update_tx.send(self.version());
            }
            Err(e) => {
                log::error!(
                    "event=error msg=API_DISCOVERY_FAILED recovery=keep_last_state err={}",
                    e
                );
            }
        }
    }

    fn update(&self) {
        // Prevent spawning multiple concurrent update loops if several pipelines share
        // the same discovery instance (e.g. grouped by scrape interval).
        if self.update_started.swap(true, Ordering::SeqCst) {
            log::debug!("event=debug msg=api_discovery_update_already_started url={}", self.url);
            return;
        }

        let this = self.clone();
        tokio::spawn(async move {
            log::warn!(
                "event=warn msg=starting_api_discovery_update_task interval_ms={}",
                this.refresh_interval.to_duration().as_millis()
            );
            loop {
                match this.get_targets_from_api().await {
                    Ok(data) => {
                        log::info!("event=info msg=api_discovery_update_success url={}", this.url);
                        this.targets.write().await.clone_from(&data);
                        this.version.fetch_add(1, Ordering::Relaxed);
                        let _ = this.update_tx.send(this.version());
                    }
                    Err(e) => {
                        log::error!(
                            "event=error msg=API_DISCOVERY_UPDATE_FAILED recovery=keep_last_state err={}",
                            e
                        );
                    }
                }

                tokio::time::sleep(this.refresh_interval.to_duration()).await;
            }
        });
    }

    fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    fn subscribe(&self) -> Option<watch::Receiver<u64>> {
        Some(self.update_rx.clone())
    }

    async fn get_targets(&self) -> Vec<Self::Target> {
        self.targets.read().await.clone()
    }
}
