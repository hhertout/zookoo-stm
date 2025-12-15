use std::{collections::HashMap, sync::Arc};

use crate::resolvers::resolve_exporters;
use crate::types::{ExportersMap, ProbeType};
use crate::{Exporter, MetricData};

#[derive(Default)]
struct NoopExporter;

impl Exporter for NoopExporter {
    fn build(_config: &configuration::model::Configuration, _exporters: &mut ExportersMap)
    where
        Self: Sized,
    {
    }

    fn export(&self, _probe_type: ProbeType, _metric_data: MetricData) {}
}

fn exporters_with(key: &str) -> ExportersMap {
    let mut exporters: ExportersMap = HashMap::new();
    exporters.insert(key.to_string(), Arc::new(NoopExporter));
    exporters
}

#[test]
fn resolve_exporters_resolves_plain_reference() {
    let exporters = exporters_with("exporter.otlp.main");
    let resolved = resolve_exporters(&["exporter.otlp.main".to_string()], &exporters);
    assert_eq!(resolved.len(), 1);
}

#[test]
fn resolve_exporters_strips_wrapper_syntax() {
    let exporters = exporters_with("exporter.otlp.main");
    let resolved = resolve_exporters(&["${exporter.otlp.main}".to_string()], &exporters);
    assert_eq!(resolved.len(), 1);
}

#[test]
#[should_panic(expected = "No forward_to specified for exporters")]
fn resolve_exporters_panics_on_empty_forward_to() {
    let exporters: ExportersMap = HashMap::new();
    let _ = resolve_exporters(&[], &exporters);
}

#[test]
#[should_panic(expected = "Exporter not found for reference")]
fn resolve_exporters_panics_on_unknown_exporter() {
    let exporters: ExportersMap = HashMap::new();
    let _ = resolve_exporters(&["exporter.otlp.missing".to_string()], &exporters);
}
