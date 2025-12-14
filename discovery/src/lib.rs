pub(crate) mod api;
pub(crate) mod file;

use async_trait::async_trait;
use configuration::model::discovery::{DiscoveryApi, DiscoveryFile};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::watch;

use crate::{api::ApiDiscovery, file::FileDiscovery};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryType {
    File,
    Api,
}

impl From<&str> for DiscoveryType {
    fn from(s: &str) -> Self {
        match s {
            "file" => DiscoveryType::File,
            "api" => DiscoveryType::Api,
            _ => panic!("Unknown discovery type: {}", s),
        }
    }
}

#[async_trait]
pub trait Discovery: Send + Sync {
    /// Target associated type (e.g. a struct describing an HTTP target or an ICMP target)
    type Target: Clone + std::fmt::Debug + Send + Sync + 'static;

    /// Return current targets. The Discovery is already specialized for a specific probe type.
    async fn discover(&self);

    /// Return current targets.
    async fn get_targets(&self) -> Vec<Self::Target>;

    /// Refresh discovery state (async operation).
    fn update(&self) {}

    /// Monotonic version number for the cached targets.
    ///
    /// Pipelines can use this to avoid re-reading/re-grouping targets on every scrape
    /// when the underlying discovery data hasn't changed.
    fn version(&self) -> u64 {
        0
    }

    /// Subscribe to target updates.
    ///
    /// Implementations that can update over time (e.g. API discovery) should return a receiver
    /// that changes whenever the cached targets change.
    fn subscribe(&self) -> Option<watch::Receiver<u64>> {
        None
    }
}

pub async fn build_discovery_file<T>(config: DiscoveryFile) -> Arc<dyn Discovery<Target = T>>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    let file_discovery = FileDiscovery::<T>::new(config);
    file_discovery.discover().await;

    Arc::new(file_discovery)
}

pub async fn build_discovery_api<T>(config: DiscoveryApi) -> Arc<dyn Discovery<Target = T>>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    let api_discovery = ApiDiscovery::<T>::new(config);
    api_discovery.discover().await;

    Arc::new(api_discovery)
}
