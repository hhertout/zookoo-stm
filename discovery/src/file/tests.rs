#[cfg(test)]
mod file_tests {
    use super::super::FileDiscovery;
    use crate::Discovery;
    use configuration::model::discovery::DiscoveryFile;
    use serde::{Deserialize, Serialize};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestHttpTarget {
        url: String,
        name: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestIcmpTarget {
        ip: String,
        name: String,
    }

    fn create_temp_json_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        file
    }

    fn file_conf(path: impl Into<String>) -> DiscoveryFile {
        DiscoveryFile { path: path.into(), labels: None, scrape_interval: None, probe_type: None }
    }

    #[tokio::test]
    async fn test_file_discovery_with_valid_file() {
        let json_content = r#"[{"url": "https://example.com", "name": "Test"}]"#;
        let temp_file = create_temp_json_file(json_content);
        let discovery = FileDiscovery::<TestHttpTarget>::new(file_conf(
            temp_file.path().to_string_lossy().to_string(),
        ));

        // new() no longer loads targets
        assert!(discovery.get_targets().await.is_empty());

        discovery.discover().await;
        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 1, "Should load targets from valid file");
    }

    #[tokio::test]
    async fn test_file_discovery_load_http_targets() {
        let json_content = r#"[
            {"url": "https://example.com", "name": "Example"},
            {"url": "https://test.com", "name": "Test"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_conf(
            file_path.to_string_lossy().to_string(),
        ));

        discovery.discover().await;

        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].url, "https://example.com");
        assert_eq!(targets[0].name, "Example");
        assert_eq!(targets[1].url, "https://test.com");
        assert_eq!(targets[1].name, "Test");
    }

    #[tokio::test]
    async fn test_file_discovery_load_icmp_targets() {
        let json_content = r#"[
            {"ip": "8.8.8.8", "name": "Google DNS"},
            {"ip": "1.1.1.1", "name": "Cloudflare DNS"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestIcmpTarget>::new(file_conf(
            file_path.to_string_lossy().to_string(),
        ));

        discovery.discover().await;

        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].ip, "8.8.8.8");
        assert_eq!(targets[0].name, "Google DNS");
        assert_eq!(targets[1].ip, "1.1.1.1");
        assert_eq!(targets[1].name, "Cloudflare DNS");
    }

    #[tokio::test]
    async fn test_file_discovery_empty_file() {
        let json_content = "[]";

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_conf(
            file_path.to_string_lossy().to_string(),
        ));

        discovery.discover().await;

        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 0, "Empty JSON should result in no targets");
    }

    #[tokio::test]
    #[should_panic(expected = "Error while reading file")]
    async fn test_file_discovery_nonexistent_file() {
        let discovery = FileDiscovery::<TestHttpTarget>::new(file_conf(
            "/nonexistent/path/to/file.json".to_string(),
        ));
        discovery.discover().await;
    }

    #[tokio::test]
    async fn test_file_discovery_update_method() {
        let json_content = r#"[
            {"url": "https://example.com", "name": "Example"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_conf(
            file_path.to_string_lossy().to_string(),
        ));

        // Load once
        discovery.discover().await;
        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://example.com");

        // update() is currently a no-op for FileDiscovery (default trait impl)
        discovery.update();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 1);
    }

    #[tokio::test]
    async fn test_file_discovery_concurrent_reads() {
        let json_content = r#"[
            {"url": "https://example.com", "name": "Example"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_conf(
            file_path.to_string_lossy().to_string(),
        ));

        discovery.discover().await;

        // Spawn multiple concurrent reads of cached targets
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let disc = discovery.clone();
                tokio::spawn(async move {
                    let targets = disc.get_targets().await;
                    assert_eq!(targets.len(), 1);
                })
            })
            .collect();

        // Wait for all to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }
    }

    #[tokio::test]
    async fn test_file_discovery_targets_loaded_at_creation() {
        // FileDiscovery now initializes empty and loads on discover().
        let json_content_v1 = r#"[
            {"url": "https://v1.com", "name": "Version 1"}
        ]"#;

        let temp_file = create_temp_json_file(json_content_v1);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_conf(
            file_path.to_string_lossy().to_string(),
        ));

        assert!(discovery.get_targets().await.is_empty());

        discovery.discover().await;
        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://v1.com");

        // Update file content - discover().await won't see the changes without explicit reload
        let json_content_v2 = r#"[
            {"url": "https://v2.com", "name": "Version 2"},
            {"url": "https://v3.com", "name": "Version 3"}
        ]"#;
        std::fs::write(&file_path, json_content_v2).expect("Failed to update file");

        // Targets are cached, so without calling discover() again we still get the original values
        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://v1.com");

        // Reload from disk
        discovery.discover().await;
        let targets = discovery.get_targets().await;
        assert_eq!(targets.len(), 2);
        assert_eq!(targets[0].url, "https://v2.com");
        assert_eq!(targets[1].url, "https://v3.com");
    }

    #[tokio::test]
    async fn test_file_discovery_clone_shares_state() {
        let json_content = r#"[
            {"url": "https://example.com", "name": "Example"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery1 = FileDiscovery::<TestHttpTarget>::new(file_conf(
            file_path.to_string_lossy().to_string(),
        ));

        // Clone should share the same Arc<RwLock<Vec<T>>>
        let discovery2 = discovery1.clone();

        discovery1.discover().await;
        let targets1 = discovery1.get_targets().await;
        let targets2 = discovery2.get_targets().await;

        assert_eq!(targets1.len(), 1);
        assert_eq!(targets2.len(), 1);
        assert_eq!(targets1[0].url, targets2[0].url);
    }
}
