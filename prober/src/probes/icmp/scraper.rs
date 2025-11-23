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
    core::{MetricExportable, ScrapeError, Scraping},
    get_tracer,
    probes::icmp::{
        metrics::IcmpRequestMetrics,
        ping::{IcmpMetrics, ping_target},
    },
    tracing_new_span,
};

#[derive(PartialEq, Copy, Clone)]
pub enum TargetType {
    IPV4,
}

impl ToString for TargetType {
    fn to_string(&self) -> String {
        match self {
            TargetType::IPV4 => String::from("ipv4"),
        }
    }
}

#[derive(Clone)]
pub struct IcmpScraper {
    pub targets: Vec<IcmpTarget>,
}

impl Scraping<IcmpTarget> for IcmpScraper {
    fn new(targets: Vec<IcmpTarget>) -> Self {
        IcmpScraper { targets }
    }

    async fn scrape(&self) -> Result<(), ScrapeError> {
        let mut span = tracing_new_span(get_tracer(), "scrape_target".to_string());
        span.set_attribute(KeyValue::new("type", "icmp".to_string()));
        let ctx = Context::current_with_span(span);
        let guard = ctx.clone().attach();

        let futures = self.targets.iter().map(|target| {
            let ctx = ctx.clone();
            self.send_request(target, ctx)
        });

        drop(guard);
        let _ = join_all(futures).await;

        Ok(())
    }

    async fn send_request(&self, target: &IcmpTarget, ctx: Context) -> Result<(), ScrapeError> {
        let span_attr = vec![
            KeyValue::new("ipv4", target.ipv4.clone().unwrap_or("unset".to_string())),
            KeyValue::new("fqdn", target.fqdn.clone().unwrap_or("unset".to_string())),
        ];
        let ctx_with_span = child_span_from_context("send_request", ctx.clone(), span_attr);

        let kind = match self.get_target_type(target) {
            Ok(target_type) => target_type,
            Err(err) => {
                log::error!("{}", err.to_string());
                let span_ref = ctx_with_span.span();
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
            .build_icmp_metrics(kind, target, ctx_with_span.clone())
            .await
        {
            let span_ref = ctx_with_span.span();
            span_ref.set_status(Status::Ok);
            log::info!(
                "event=metrics ipv4={} fqdn={} job=zookoo {}",
                target.ipv4.clone().unwrap_or("unset".to_string()),
                target.fqdn.clone().unwrap_or("unset".to_string()),
                metrics.to_logfmt()
            );

            self.export_metrics(
                kind,
                metrics.target.clone(),
                IcmpRequestMetrics {
                    up: metrics.up,
                    duration: metrics.duration,
                    labels: target.labels.clone(),
                },
                ctx_with_span,
            );
        } else {
            let span_ref = ctx_with_span.span();
            log::error!("build metrics failed");
            span_ref.set_status(Status::Error {
                description: std::borrow::Cow::Borrowed("probe failed"),
            });
        }

        return Ok(());
    }
}

impl IcmpScraper {
    async fn build_icmp_metrics(
        &self,
        _: TargetType,
        target: &IcmpTarget,
        ctx: Context,
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
        let ctx_with_span = child_span_from_context("build_icmp_metrics", ctx.clone(), span_attr);
        let span_ref = ctx_with_span.span();

        // todo
        match ping_target(target, ctx_with_span.clone()).await {
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

    fn export_metrics(&self, _kind: TargetType, target: String, metrics: IcmpRequestMetrics, ctx: Context) {
        let span_attr = vec![KeyValue::new("target", target.clone())];
        let ctx_with_span = child_span_from_context("export_metrics", ctx.clone(), span_attr);

        metrics.export(&target);

        let span_ref = ctx_with_span.span();
        span_ref.set_status(Status::Ok);
    }

    fn get_target_type(&self, _: &IcmpTarget) -> Result<TargetType, Error> {
        Ok(TargetType::IPV4)
    }
}
