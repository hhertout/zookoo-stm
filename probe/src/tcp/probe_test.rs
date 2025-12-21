use configuration::model::target::TcpTarget;

use super::probe::TcpProbe;

use crate::MetricData;
use crate::Probe;

#[test]
fn test_tcp_target_addr() {
    let t = TcpTarget { target: "127.0.0.1:8080".to_string(), labels: None, timeout_sec: 5 };
    assert_eq!(t.target, "127.0.0.1:8080");
}

#[tokio::test]
async fn test_tcp_probe_connect_success() {
    // Start a short-lived TCP listener
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().unwrap();

    // accept one connection in background
    tokio::spawn(async move {
        if let Ok((mut _socket, _peer)) = listener.accept().await {
            // hold the connection briefly
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    let mut probe = TcpProbe::init("tcp_test_probe".to_string());
    probe.set_targets(vec![TcpTarget {
        target: format!("{}", addr),
        labels: None,
        timeout_sec: 5,
    }]);

    probe.scrape().await;

    let metrics = probe.get_metrics().await;
    assert_eq!(metrics.len(), 1);
    let m: &MetricData = &metrics[0];
    let up = m.metrics.get("up").copied().unwrap_or(0);
    assert_eq!(up, 1);
}

#[tokio::test]
async fn test_tcp_probe_connect_failure() {
    // Pick an unused port (0 can't be used; use high port likely unused)
    let port = 59999u16;

    let mut probe = TcpProbe::init("tcp_test_probe_fail".to_string());
    probe.set_targets(vec![TcpTarget {
        target: format!("127.0.0.1:{}", port),
        labels: None,
        timeout_sec: 5,
    }]);

    probe.scrape().await;

    let metrics = probe.get_metrics().await;
    assert_eq!(metrics.len(), 1);
    let m: &MetricData = &metrics[0];
    let up = m.metrics.get("up").copied().unwrap_or(1);
    assert_eq!(up, 0);
}

#[tokio::test]
async fn test_tcp_probe_connect_success_fqdn_localhost() {
    // Start a short-lived TCP listener bound to loopback
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().unwrap();

    // accept one connection in background
    tokio::spawn(async move {
        if let Ok((mut _socket, _peer)) = listener.accept().await {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    });

    let mut probe = TcpProbe::init("tcp_test_probe_localhost".to_string());
    probe.set_targets(vec![TcpTarget {
        target: format!("localhost:{}", addr.port()),
        labels: None,
        timeout_sec: 5,
    }]);

    probe.scrape().await;

    let metrics = probe.get_metrics().await;
    assert_eq!(metrics.len(), 1);
    let m: &MetricData = &metrics[0];
    let up = m.metrics.get("up").copied().unwrap_or(0);
    assert_eq!(up, 1);
}

#[tokio::test]
async fn test_tcp_probe_connect_failure_dns() {
    // Use a guaranteed non-resolving domain (.invalid)
    let mut probe = TcpProbe::init("tcp_test_probe_dns_fail".to_string());
    probe.set_targets(vec![TcpTarget {
        target: "nonexistent-domain.invalid:12345".to_string(),
        labels: None,
        timeout_sec: 2,
    }]);

    probe.scrape().await;

    let metrics = probe.get_metrics().await;
    assert_eq!(metrics.len(), 1);
    let m: &MetricData = &metrics[0];
    let up = m.metrics.get("up").copied().unwrap_or(1);
    assert_eq!(up, 0);
}

#[tokio::test]
async fn test_tcp_probe_connect_failure_no_port() {
    // Provide a host without port — should fail to connect
    let mut probe = TcpProbe::init("tcp_test_probe_no_port".to_string());
    probe.set_targets(vec![TcpTarget {
        target: "localhost".to_string(),
        labels: None,
        timeout_sec: 2,
    }]);

    probe.scrape().await;

    let metrics = probe.get_metrics().await;
    assert_eq!(metrics.len(), 1);
    let m: &MetricData = &metrics[0];
    let up = m.metrics.get("up").copied().unwrap_or(1);
    assert_eq!(up, 0);
}
