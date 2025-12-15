use std::sync::Arc;

use crate::{Exporter, types::ExportersMap};

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
