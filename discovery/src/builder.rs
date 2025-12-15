use serde::de::DeserializeOwned;
use std::sync::Arc;

use crate::{Discovery, DiscoveryType, api::ApiDiscovery, file::FileDiscovery};

/// Build a Discovery implementation from a DiscoveryType configuration.
/// The returned Discovery is already initialized with initial targets discovered.
pub async fn build_discovery<T>(config: DiscoveryType) -> Arc<dyn Discovery<Target = T>>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    match config {
        DiscoveryType::File(conf) => {
            let discovery = FileDiscovery::<T>::new(conf);
            // auto discover initial targets
            discovery.discover().await;

            Arc::new(discovery)
        }
        DiscoveryType::Api(conf) => {
            let discovery = ApiDiscovery::<T>::new(conf);
            // auto discover initial targets
            discovery.discover().await;

            Arc::new(discovery)
        }
    }
}
