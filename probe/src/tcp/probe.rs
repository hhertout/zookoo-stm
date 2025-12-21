use std::{
    collections::HashMap,
    fmt::Display,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::future::join_all;
use tokio::{net::TcpStream, sync::Mutex};

use tracing::{Instrument, info_span};

use configuration::{DEFAULT_SOURCE, model::target::TcpTarget};

use crate::{MetricData, Probe};

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    TCP,
}

impl Display for TargetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetType::TCP => write!(f, "TCP"),
        }
    }
}

#[derive(Clone)]
pub struct TcpProbe {
    name: String,
    targets: Vec<TcpTarget>,
    metrics: Arc<Mutex<Vec<MetricData>>>,
}

impl TcpProbe {}

impl Probe for TcpProbe {
    type Target = TcpTarget;

    fn init(name: String) -> Self {
        TcpProbe { name, targets: Vec::new(), metrics: Arc::new(Mutex::new(Vec::new())) }
    }

    fn set_targets(&mut self, targets: Vec<Self::Target>) {
        self.targets = targets;
    }

    fn get_metrics(&self) -> impl std::future::Future<Output = Vec<MetricData>> + Send {
        let metrics = Arc::clone(&self.metrics);
        async move {
            let mut guard = metrics.lock().await;
            let result = guard.clone();
            guard.clear();
            result
        }
        .instrument(info_span!("tcp.get_metrics"))
    }

    fn scrape(&self) -> impl std::future::Future<Output = ()> + Send {
        let targets = self.targets.clone();
        let metrics = Arc::clone(&self.metrics);
        let name = self.name.clone();

        async move {
            let futures = targets.into_iter().map(|target| {
                let metrics = Arc::clone(&metrics);
                let name = name.clone();
                async move {
                    log::info!(
                        "source={} probe={} type={} target={} event=request_start",
                        DEFAULT_SOURCE,
                        name,
                        "tcp",
                        target.target
                    );

                    let start = Instant::now();
                    let up = match tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&target.target)).await {
                        Ok(Ok(_stream)) => 1,
                        Ok(Err(e)) => {
                            log::error!("source={} probe={} type={} target={} event=connect_failed err={}", DEFAULT_SOURCE, name, "tcp", target.target, e);
                            0
                        }
                        Err(_) => {
                            log::error!("source={} probe={} type={} target={} event=connect_timeout", DEFAULT_SOURCE, name, "tcp", target.target);
                            0
                        }
                    };

                    let duration_ms = start.elapsed().as_millis() as isize;
                    let duration_seconds = (duration_ms as f64) / 1000.0;

                    log::info!(
                        "source={} probe={} type={} target={} event=request_complete duration_seconds={}",
                        DEFAULT_SOURCE,
                        name,
                        "tcp",
                        target.target,
                        duration_seconds
                    );

                    // Build metrics
                    let mut metrics_map = HashMap::new();
                    metrics_map.insert("up".to_string(), up as isize);
                    metrics_map.insert("rtt_ms".to_string(), duration_ms);


                    let metric_data = MetricData::with_metrics(metrics_map)
                        .with_labels(target.labels.clone())
                        .with_probe(crate::ProbeType::Tcp)
                        .with_instance(target.target);

                    let mut guard = metrics.lock().await;
                    guard.push(metric_data);
                }
            });

            let _ = join_all(futures).await;
        }
    }
}
