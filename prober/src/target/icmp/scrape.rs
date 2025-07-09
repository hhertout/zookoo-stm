use std::io::Error;

use futures::future::join_all;
use opentelemetry::{
    Context, KeyValue,
    global::ObjectSafeSpan,
    trace::{Status, TraceContextExt},
};

use crate::{
    child_span_from_context,
    config::target::IcmpTarget,
    get_tracer,
    metrics::{Metrics, icmp_metrics::IcmpRequestMetrics},
    target::{ScrapeError, Scraping, TargetType, icmp::ping::IcmpMetrics},
    tracing_new_span,
};
use crate::{metrics::MetricExportable, target::icmp::ping::ping_target};

#[derive(Clone)]
pub struct IcmpScrapper {
    pub targets: Vec<IcmpTarget>,
}

impl Scraping<IcmpTarget> for IcmpScrapper {
    fn new(targets: Vec<IcmpTarget>) -> Self {
        IcmpScrapper { targets }
    }

    async fn scrape(&self) -> Result<(), ScrapeError> {
        let mut span = tracing_new_span(get_tracer(), "scrape_target".to_string());
        span.set_attribute(KeyValue::new("type", "icmp".to_string()));
        let cx = Context::current_with_span(span);
        let guard = cx.clone().attach();

        let futures = self.targets.iter().map(|target| {
            let ctx = cx.clone();
            self.send_request(target, ctx)
        });

        drop(guard);
        let _ = join_all(futures).await;

        Ok(())
    }

    async fn send_request(&self, target: &IcmpTarget, cx: Context) -> Result<(), ScrapeError> {
        let span_attr = vec![
            KeyValue::new("ipv4", target.ipv4.clone().unwrap_or("unset".to_string())),
            KeyValue::new("fqdn", target.fqdn.clone().unwrap_or("unset".to_string())),
        ];
        let cx_with_span = child_span_from_context("send_request", cx.clone(), span_attr);

        let kind = match self.get_target_type(target) {
            Ok(target_type) => target_type,
            Err(err) => {
                log::error!("{}", err.to_string());
                let span_ref = cx_with_span.span();
                span_ref.set_status(Status::Error {
                    description: "get_target_type failed".into(),
                });
                span_ref.end();
                return Err(ScrapeError::TypeError(err.to_string()));
            }
        };

        log::info!(
            "event=request type={} ipv4={} fqdn={}",
            kind.to_string(),
            target.ipv4.clone().unwrap_or("unset".to_string()),
            target.fqdn.clone().unwrap_or("unset".to_string())
        );

        if let Some(metrics) = self
            .build_icmp_metrics(kind, target, cx_with_span.clone())
            .await
        {
            let span_ref = cx_with_span.span();
            span_ref.set_status(Status::Ok);
            log::info!(
                "event=metrics ipv4={} fqdn={} job=zookoo {}",
                target.ipv4.clone().unwrap_or("unset".to_string()),
                target.fqdn.clone().unwrap_or("unset".to_string()),
                metrics.to_logfmt()
            );

            self.export_metrics(
                kind,
                metrics.target,
                Metrics::Icmp(IcmpRequestMetrics {
                    up: metrics.up,
                    duration: metrics.duration,
                    labels: target.labels.clone(),
                }),
                cx_with_span,
            );
        } else {
            let span_ref = cx_with_span.span();
            log::error!("build metrics failed");
            span_ref.set_status(Status::Error {
                description: std::borrow::Cow::Borrowed("probe failed"),
            });
        }

        return Ok(());
    }
}

impl IcmpScrapper {
    async fn build_icmp_metrics(
        &self,
        _: TargetType,
        target: &IcmpTarget,
        cx: Context,
    ) -> Option<IcmpMetrics> {
        let span_attr = vec![
            KeyValue::new(
                "ipv4",
                target.ipv4.clone().unwrap_or("unset".to_string()).clone(),
            ),
            KeyValue::new(
                "fqdn",
                target.fqdn.clone().unwrap_or("unset".to_string()).clone(),
            ),
        ];
        let cx_with_span = child_span_from_context("build_icmp_metrics", cx.clone(), span_attr);
        let span_ref = cx_with_span.span();

        // todo
        match ping_target(target, cx_with_span.clone()).await {
            Ok((ip, duration)) => {
                span_ref.set_status(Status::Ok);
                Some(IcmpMetrics {
                    target: target.fqdn.clone().unwrap_or(ip.to_string()), // set the fqdn or the ip if it is None
                    up: 1,
                    duration: duration,
                })
            }
            Err(err) => {
                log::error!("ipv4={:?} fqdn={:?} err={}", target.ipv4, target.fqdn, err);
                span_ref.set_status(Status::Error {
                    description: err.to_string().into(),
                });
                None
            }
        }
    }

    fn export_metrics(&self, kind: TargetType, target: String, metrics: Metrics, cx: Context) {
        let span_attr = vec![KeyValue::new("target", target.clone())];
        let cx_with_span = child_span_from_context("export_metrics", cx.clone(), span_attr);

        match (kind, metrics) {
            (TargetType::IPV4, Metrics::Icmp(m)) => m.export(&target),
            _ => {
                log::error!(
                    "wrong exporter type, got {}, expect ipv4 or fqdn",
                    kind.to_string()
                )
            }
        };

        let span_ref = cx_with_span.span();
        span_ref.set_status(Status::Ok);
    }

    fn get_target_type(&self, _: &IcmpTarget) -> Result<TargetType, Error> {
        Ok(TargetType::IPV4)
    }
}
