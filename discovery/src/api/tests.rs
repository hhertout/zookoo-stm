use std::collections::HashMap;

use configuration::model::RefreshInterval;
use configuration::model::discovery::DiscoveryApi;
use httpmock::Method::GET;
use serde::Deserialize;
use tokio::time::{Duration, advance};

use crate::Discovery;

use super::ApiDiscovery;

#[test]
fn to_headers_map_includes_custom_headers() {
    let mut headers = HashMap::new();
    headers.insert("x-test".to_string(), "value1".to_string());
    headers.insert("x-other".to_string(), "value2".to_string());

    let discovery: ApiDiscovery<()> = ApiDiscovery::new(DiscoveryApi {
        url: "https://example.invalid".to_string(),
        headers: Some(headers),
        basic_auth: None,
        refresh_interval: configuration::model::RefreshInterval::H1,
        bearer: None,
    });

    let headers = discovery.to_headers_map();

    assert_eq!(headers.get("x-test").unwrap().to_str().unwrap(), "value1");
    assert_eq!(headers.get("x-other").unwrap().to_str().unwrap(), "value2");
    assert!(headers.get(reqwest::header::AUTHORIZATION).is_none());
}

#[test]
fn to_headers_map_adds_authorization_when_basic_auth_provided() {
    let mut headers = HashMap::new();
    headers.insert("x-test".to_string(), "value".to_string());

    let discovery: ApiDiscovery<()> = ApiDiscovery::new(DiscoveryApi {
        url: "https://example.invalid".to_string(),
        headers: Some(headers),
        basic_auth: Some("test-token".to_string()),
        bearer: None,
        refresh_interval: configuration::model::RefreshInterval::H1,
    });

    let headers = discovery.to_headers_map();

    assert_eq!(headers.get("x-test").unwrap().to_str().unwrap(), "value");
    assert_eq!(
        headers.get(reqwest::header::AUTHORIZATION).unwrap().to_str().unwrap(),
        "Basic test-token"
    );
}

#[test]
fn to_headers_map_bearer_assignation_test() {
    let mut headers = HashMap::new();
    headers.insert("authorization".to_string(), "Bearer from-headers".to_string());

    let discovery: ApiDiscovery<()> = ApiDiscovery::new(DiscoveryApi {
        url: "https://example.invalid".to_string(),
        headers: Some(headers),
        basic_auth: None,
        bearer: Some("from-bearer".to_string()),
        refresh_interval: configuration::model::RefreshInterval::H1,
    });

    let headers = discovery.to_headers_map();

    assert_eq!(
        headers.get(reqwest::header::AUTHORIZATION).unwrap().to_str().unwrap(),
        "Bearer from-bearer"
    );
}

#[test]
fn to_headers_map_authorization_overrides_existing_authorization_header() {
    let mut headers = HashMap::new();
    headers.insert("authorization".to_string(), "Bearer from-headers".to_string());

    let discovery: ApiDiscovery<()> = ApiDiscovery::new(DiscoveryApi {
        url: "https://example.invalid".to_string(),
        headers: Some(headers),
        basic_auth: Some("from-basic-auth".to_string()),
        bearer: None,
        refresh_interval: configuration::model::RefreshInterval::H1,
    });

    let headers = discovery.to_headers_map();

    assert_eq!(
        headers.get(reqwest::header::AUTHORIZATION).unwrap().to_str().unwrap(),
        "Basic from-basic-auth"
    );
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
struct TestTarget {
    url: String,
    name: String,
}

fn api_conf(url: impl Into<String>, refresh_interval: RefreshInterval) -> DiscoveryApi {
    DiscoveryApi {
        url: url.into(),
        headers: None,
        basic_auth: None,
        bearer: None,
        refresh_interval,
    }
}

#[tokio::test(start_paused = true)]
async fn api_discovery_update_updates_targets_and_notifies_version() {
    let server = httpmock::MockServer::start_async().await;
    let path = "/targets";

    // First iteration returns v1, second iteration returns v2.
    let v1_mock = server
        .mock_async(|when, then| {
            when.method(GET).path(path);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{"url":"https://v1","name":"v1"}]"#);
        })
        .await;

    let _v2_mock = server
        .mock_async(|when, then| {
            when.method(GET).path(path);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{"url":"https://v2","name":"v2"},{"url":"https://v3","name":"v3"}]"#);
        })
        .await;

    let discovery: ApiDiscovery<TestTarget> =
        ApiDiscovery::new(api_conf(format!("{}{}", server.base_url(), path), RefreshInterval::S5));

    let mut rx = discovery.subscribe().expect("ApiDiscovery must expose watch receiver");

    discovery.update();
    tokio::task::yield_now().await;

    rx.changed().await.expect("watch channel should stay open");
    assert_eq!(discovery.version(), 1);
    assert_eq!(
        discovery.get_targets().await,
        vec![TestTarget { url: "https://v1".to_string(), name: "v1".to_string() }]
    );

    // Disable the first mock so the next request is served by the v2 mock.
    v1_mock.delete_async().await;

    // Next loop tick.
    advance(Duration::from_secs(5)).await;
    tokio::task::yield_now().await;

    rx.changed().await.expect("watch channel should stay open");
    assert_eq!(discovery.version(), 2);
    assert_eq!(
        discovery.get_targets().await,
        vec![
            TestTarget { url: "https://v2".to_string(), name: "v2".to_string() },
            TestTarget { url: "https://v3".to_string(), name: "v3".to_string() }
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn api_discovery_update_is_idempotent_and_does_not_spawn_multiple_loops() {
    let server = httpmock::MockServer::start_async().await;
    let path = "/targets";

    let mock = server
        .mock_async(|when, then| {
            when.method(GET).path(path);
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"[{"url":"https://same","name":"same"}]"#);
        })
        .await;

    let discovery: ApiDiscovery<TestTarget> =
        ApiDiscovery::new(api_conf(format!("{}{}", server.base_url(), path), RefreshInterval::S5));

    let mut rx = discovery.subscribe().expect("ApiDiscovery must expose watch receiver");

    // Calling update() twice must not cause two immediate fetches.
    discovery.update();
    discovery.update();

    // Wait for the first successful refresh.
    rx.changed().await.expect("watch channel should stay open");
    assert_eq!(discovery.version(), 1, "only one update loop should have incremented the version");

    let hits_after_start = mock.calls_async().await as u64;
    assert_eq!(hits_after_start, 1, "update() should spawn only one background loop");

    // Next tick should produce exactly one additional fetch.
    advance(Duration::from_secs(5)).await;
    tokio::task::yield_now().await;

    rx.changed().await.expect("watch channel should stay open");

    let hits_after_one_tick = mock.calls_async().await as u64;
    assert_eq!(hits_after_one_tick, 2);
}
