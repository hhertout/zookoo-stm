use std::{fmt::Display, sync::Arc};

use configuration::model::target::IcmpTarget;
use futures::future::join_all;
use tokio::sync::Mutex;
use tracing::{Instrument, info_span};

use crate::{MetricData, Probe, icmp::ping::ping_target};

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    IPV4,
    FQDN,
}

impl Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::IPV4 => write!(f, "IPV4"),
            TargetType::FQDN => write!(f, "FQDN"),
        }
    }
}

#[derive(Clone)]
pub struct IcmpProbe {
    targets: Option<Vec<IcmpTarget>>,
    metrics: Arc<Mutex<Vec<MetricData>>>,
}

impl IcmpProbe {}

impl Probe for IcmpProbe {
    type Target = IcmpTarget;

    #[tracing::instrument(level = "debug")]
    fn init() -> Self {
        IcmpProbe { targets: None, metrics: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Set or update the target data for this probe.
    #[tracing::instrument(level = "debug", skip(self, targets), fields(target_count = targets.len()))]
    fn set_targets(&mut self, targets: Vec<Self::Target>) {
        self.targets = Some(targets);
    }

    fn get_metrics(&self) -> impl std::future::Future<Output = Vec<MetricData>> + Send {
        let metrics = Arc::clone(&self.metrics);
        async move {
            let mut guard = metrics.lock().await;
            let result = guard.clone();
            guard.clear();
            result
        }
        .instrument(info_span!("icmp.get_metrics"))
    }

    #[tracing::instrument(level = "debug", skip(self), fields(target_count = self.targets.as_ref().map(|t| t.len()).unwrap_or(0)))]
    async fn scrape(&self) {
        let targets = self.targets.as_ref().unwrap();

        let futures = targets.iter().map(|target| {
            let metrics = Arc::clone(&self.metrics);

            let ipv4 = target.ipv4.clone().unwrap_or_else(|| "unset".to_string());
            let fqdn = target.fqdn.clone().unwrap_or_else(|| "unset".to_string());
            let timeout_sec = target.timeout_sec;
            let span = info_span!(
                "icmp.scrape_target",
                ipv4 = %ipv4,
                fqdn = %fqdn,
                timeout_sec = timeout_sec,
            );

            async move {
                let (up, duration_ms) = match ping_target(target).await {
                    Ok((ip, duration)) => {
                        log::info!(
                            "event=ping_success target={} up=1 duration_ms={}",
                            ip,
                            duration.as_millis()
                        );
                        (1, duration.as_millis())
                    }
                    Err(e) => {
                        log::error!("event=ping_failed err={}", e);
                        (0, 0)
                    }
                };

                let instance = if let Some(fqdn) = &target.fqdn {
                    fqdn.clone()
                } else if let Some(ipv4) = &target.ipv4 {
                    ipv4.clone()
                } else {
                    "unset".to_string()
                };

                // Build metrics with target labels
                let mut metrics_map = std::collections::HashMap::new();
                metrics_map.insert("up".to_string(), up as isize);
                metrics_map.insert("rtt_ms".to_string(), duration_ms as isize);

                let metric_data = MetricData::with_metrics(metrics_map)
                    .with_labels(target.labels.clone())
                    .with_probe(crate::ProbeType::Icmp)
                    .with_instance(instance);

                let mut metrics_lock = metrics.lock().await;
                metrics_lock.push(metric_data);
            }
            .instrument(span)
        });

        let _ = join_all(futures).await;
    }
}
