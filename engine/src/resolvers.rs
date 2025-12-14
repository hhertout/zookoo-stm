use std::sync::Arc;

use configuration::model::Configuration;
use exporter::Exporter;

use crate::ExportersMap;

/// Resolve a file discovery reference like "discovery.file.json_targets" or "${discovery.file.json_targets}"
pub async fn resolve_discovery<T>(
    reference: &str,
    config: &Configuration,
) -> Option<Arc<dyn discovery::Discovery<Target = T> + Send + Sync>>
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
                return Some(discovery::build_discovery_file(file_config.clone()).await);
            }
            None
        }
        (Some(&"discovery"), Some(&"api"), Some(label)) => {
            if let Some(ref discovery_wrapper) = config.discovery
                && let Some(api_config) = discovery_wrapper.api.get(*label)
            {
                return Some(discovery::build_discovery_api(api_config.clone()).await);
            }
            None
        }
        _ => None,
    }
}

/// Resolve exporters from forward_to references
/// References can be like "exporter.otlp.main" or "${exporter.otlp.main}"
pub fn resolve_exporters(
    forward_to: &[String],
    all_exporters: &ExportersMap,
) -> Vec<Arc<dyn Exporter + Send + Sync>> {
    if forward_to.is_empty() {
        // If no forward_to specified, throw an error and panic
        log::error!("event=error msg=no_forward_to_specified_for_exporters");
        log::error!("INVALID CONFIGURATION");
        log::error!(
            "Unrecoverable error: No forward_to specified for exporters ! Key forward_to is mandatory in probe configuration."
        );
        panic!("No forward_to specified for exporters");
    }

    let mut resolved = Vec::new();
    for reference in forward_to {
        // Strip ${} wrapper if present
        let key =
            reference.strip_prefix("${").and_then(|s| s.strip_suffix("}")).unwrap_or(reference);

        if let Some(exporter) = all_exporters.get(key) {
            resolved.push(exporter.clone());
            log::debug!("event=exporter_resolved reference={} key={}", reference, key);
        } else {
            log::error!(
                "event=exporter_not_found reference={} available={:?}",
                reference,
                all_exporters.keys().collect::<Vec<_>>()
            );
            log::error!("INVALID CONFIGURATION");
            panic!("Exporter not found for reference: {}", reference);
        }
    }

    resolved
}
