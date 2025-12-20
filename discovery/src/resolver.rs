use std::sync::Arc;

use configuration::model::Configuration;
use tokio::sync::RwLock;

use crate::{Discovery, DiscoveryType, builder::build_discovery};

/// Resolve a Discovery from a reference string and configuration.
/// References can be like "discovery.file.main" or "${discovery.api.main}"
/// Returns None if the reference cannot be resolved.
#[tracing::instrument(skip_all, fields(reference = %reference))]
pub async fn resolve_discovery<T>(
    reference: &str,
    config: &Configuration,
) -> Option<Arc<RwLock<dyn Discovery<Target = T> + Send + Sync>>>
where
    T: Clone + std::fmt::Debug + Send + Sync + serde::de::DeserializeOwned + 'static,
{
    // Strip ${} wrapper if present
    let reference =
        reference.strip_prefix("${").and_then(|s| s.strip_suffix("}")).unwrap_or(reference);

    let parts: Vec<&str> = reference.split('.').collect();
    match (parts.first(), parts.get(1), parts.get(2)) {
        (Some(&"discovery"), Some(&"file"), Some(label)) => {
            if let Some(ref discovery_wrapper) = config.discovery
                && let Some(file_config) = discovery_wrapper.file.get(*label)
            {
                tracing::trace!(kind = "discovery", r#type = "file", label = %label, "resolve_discovery");
                let discovery = build_discovery(DiscoveryType::File(file_config.clone())).await;
                return Some(discovery);
            }
            None
        }
        (Some(&"discovery"), Some(&"api"), Some(label)) => {
            if let Some(ref discovery_wrapper) = config.discovery
                && let Some(api_config) = discovery_wrapper.api.get(*label)
            {
                tracing::trace!(kind = "discovery", r#type = "api", label = %label, "resolve_discovery");
                let discovery = build_discovery(DiscoveryType::Api(api_config.clone())).await;
                return Some(discovery);
            }
            None
        }
        _ => None,
    }
}
