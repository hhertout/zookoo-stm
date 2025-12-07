use std::collections::HashMap;

use configuration::model::defaults::Defaults;

pub fn set_defaults_labels(
    defaults: &Defaults,
    override_labels: HashMap<String, String>,
) -> HashMap<String, String> {
    // set default labels
    let mut labels = HashMap::new();
    labels.insert("job".to_string(), defaults.job.clone());
    labels.insert("service_name".to_string(), defaults.service_name.clone());

    if let Some(ref zone) = defaults.probe_zone {
        labels.insert("probe_zone".to_string(), zone.clone());
    }

    // Add probe location (latitude/longitude) if configured
    if let Some(ref location) = defaults.probe_location {
        labels.insert("latitude".to_string(), location.latitude.to_string());
        labels.insert("longitude".to_string(), location.longitude.to_string());
    }

    for (key, value) in override_labels {
        labels.insert(key, value);
    }

    labels
}
