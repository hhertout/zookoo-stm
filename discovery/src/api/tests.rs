use std::collections::HashMap;

use configuration::model::discovery::DiscoveryApi;

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
