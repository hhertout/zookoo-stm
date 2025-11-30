use std::{collections::HashMap, fmt::Display, sync::Arc};

use configuration::model::target::IcmpTarget;
use futures::future::join_all;
use opentelemetry::{
    global::ObjectSafeSpan,
    trace::{Status, TraceContextExt},
};
use tokio::sync::Mutex;

use crate::{MetricData, Probe, icmp::ping::ping_target, observability::get_empty_attributes};

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

    fn init() -> Self {
        crate::span!("init".to_string(), get_empty_attributes());

        IcmpProbe { targets: None, metrics: Arc::new(Mutex::new(Vec::new())) }
    }

    /// Set or update the target data for this probe.
    fn set_targets(&mut self, targets: Vec<Self::Target>) {
        crate::span!("set_targets".to_string(), get_empty_attributes());

        self.targets = Some(targets);
    }

    fn get_metrics(&self) -> impl std::future::Future<Output = Vec<MetricData>> + Send {
        crate::span!("get_metrics".to_string(), get_empty_attributes());

        let metrics = Arc::clone(&self.metrics);
        async move {
            let mut guard = metrics.lock().await;
            let result = guard.clone();
            guard.clear(); // Clear metrics after reading
            result
        }
    }

    async fn scrape(&self) {
        let ctx = crate::span!("scrape".to_string(), get_empty_attributes());
        let guard = ctx.clone().attach();

        let futures = self.targets.as_ref().unwrap().iter().map(|target| {
            let target = target.clone();
            let metrics = Arc::clone(&self.metrics);
            async move {
                let ctx = crate::span!("scrape_target".to_string(), get_empty_attributes());

                // Send request to get metrics
                let mut attr = HashMap::new();
                attr.insert("ipv4", target.ipv4.clone().unwrap_or("unset".to_string()));
                attr.insert("fqdn", target.fqdn.clone().unwrap_or("unset".to_string()));
                let ctx_with_span = crate::child_span!(ctx, "ping_target".to_string(), attr);

                let (up, duration_ms) = match ping_target(&target, ctx_with_span.clone()).await {
                    Ok((ip, duration)) => {
                        let span_ref = ctx_with_span.span();
                        span_ref.set_status(Status::Ok);

                        log::info!(
                            "event=ping_success target={} up=1 duration_ms={}",
                            ip,
                            duration.as_millis()
                        );
                        (1, duration.as_millis())
                    }
                    Err(e) => {
                        log::error!("event=ping_failed err={}", e);
                        let span_ref = ctx_with_span.span();
                        span_ref.set_status(Status::Error {
                            description: std::borrow::Cow::Borrowed("probe failed"),
                        });
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
        });

        drop(guard);
        let _ = join_all(futures).await;
    }
}
