use serde::de::DeserializeOwned;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Discovery, DiscoveryType, api::ApiDiscovery, file::FileDiscovery};

/// Build a Discovery implementation from a DiscoveryType configuration.
/// The returned Discovery is already initialized with initial targets discovered.
#[tracing::instrument(level = "info", skip(config), fields(kind = tracing::field::Empty))]
pub async fn build_discovery<T>(config: DiscoveryType) -> Arc<RwLock<dyn Discovery<Target = T>>>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    match config {
        DiscoveryType::File(conf) => {
            tracing::Span::current().record("kind", "file");
            let discovery = FileDiscovery::<T>::new(conf);
            // auto discover initial targets
            discovery.discover().await;

            Arc::new(RwLock::new(discovery))
        }
        DiscoveryType::Api(conf) => {
            tracing::Span::current().record("kind", "api");
            let discovery = ApiDiscovery::<T>::new(conf);
            // auto discover initial targets
            discovery.discover().await;

            Arc::new(RwLock::new(discovery))
        }
    }
}
