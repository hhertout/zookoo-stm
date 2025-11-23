//! Metrics trait definition
//!
//! This module defines the core trait that all metric types must implement
//! to be exportable to various backends (OpenTelemetry, Prometheus, etc.)

/// Trait for exporting metrics to external systems
///
/// All metric types must implement this trait to be compatible with the export system.
/// The implementation should handle the conversion of the metric data to the appropriate
/// format for the configured exporter.
pub trait MetricExportable {
    /// Export the metrics for the given target
    ///
    /// # Arguments
    /// * `target` - The target identifier (URL, IP, FQDN, etc.)
    fn export(&self, target: &str);
}
