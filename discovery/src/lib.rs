pub(crate) mod api;
pub mod builder;
pub(crate) mod file;
pub mod resolver;

use async_trait::async_trait;
use configuration::model::discovery::{DiscoveryApi, DiscoveryFile};
use tokio::sync::watch;

#[cfg(test)]
mod resolvers_test;

#[derive(Debug, Clone)]
pub enum DiscoveryType {
    File(DiscoveryFile),
    Api(DiscoveryApi),
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
