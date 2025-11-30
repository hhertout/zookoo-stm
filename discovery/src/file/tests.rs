#[cfg(test)]
mod file_tests {
    use super::super::FileDiscovery;
    use crate::Discovery;
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

    #[tokio::test]
    async fn test_file_discovery_with_valid_file() {
        let json_content = r#"[{"url": "https://example.com", "name": "Test"}]"#;
        let temp_file = create_temp_json_file(json_content);
        let discovery = FileDiscovery::<TestHttpTarget>::new(temp_file.path());
        let targets = discovery.discover();
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

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_path);

        let targets = discovery.discover();
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

        let discovery = FileDiscovery::<TestIcmpTarget>::new(file_path);

        let targets = discovery.discover();
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

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_path);

        let targets = discovery.discover();
        assert_eq!(targets.len(), 0, "Empty JSON should result in no targets");
    }

    #[tokio::test]
    #[should_panic(expected = "Error while reading file")]
    async fn test_file_discovery_nonexistent_file() {
        let _discovery = FileDiscovery::<TestHttpTarget>::new("/nonexistent/path/to/file.json");
    }

    #[tokio::test]
    async fn test_file_discovery_update_method() {
        let json_content = r#"[
            {"url": "https://example.com", "name": "Example"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_path);

        // Call update (spawns async task)
        discovery.update();

        // Give it a moment to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let targets = discovery.discover();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://example.com");
    }

    #[tokio::test]
    async fn test_file_discovery_concurrent_reads() {
        let json_content = r#"[
            {"url": "https://example.com", "name": "Example"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(file_path);

        // Spawn multiple concurrent reads
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let disc = discovery.clone();
                tokio::spawn(async move {
                    let targets = disc.discover();
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
        // FileDiscovery loads targets at creation time, not on each discover() call
        let json_content_v1 = r#"[
            {"url": "https://v1.com", "name": "Version 1"}
        ]"#;

        let temp_file = create_temp_json_file(json_content_v1);
        let file_path = temp_file.path().to_path_buf();

        let discovery = FileDiscovery::<TestHttpTarget>::new(&file_path);

        let targets = discovery.discover();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://v1.com");

        // Update file content - discover() won't see the changes without explicit reload
        let json_content_v2 = r#"[
            {"url": "https://v2.com", "name": "Version 2"},
            {"url": "https://v3.com", "name": "Version 3"}
        ]"#;
        std::fs::write(&file_path, json_content_v2).expect("Failed to update file");

        // Targets are cached, so we still get the original values
        let targets = discovery.discover();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].url, "https://v1.com");
    }

    #[tokio::test]
    async fn test_file_discovery_clone_shares_state() {
        let json_content = r#"[
            {"url": "https://example.com", "name": "Example"}
        ]"#;

        let temp_file = create_temp_json_file(json_content);
        let file_path = temp_file.path().to_path_buf();

        let discovery1 = FileDiscovery::<TestHttpTarget>::new(file_path);

        // Clone should share the same Arc<RwLock<Vec<T>>>
        let discovery2 = discovery1.clone();

        let targets1 = discovery1.discover();
        let targets2 = discovery2.discover();

        assert_eq!(targets1.len(), 1);
        assert_eq!(targets2.len(), 1);
        assert_eq!(targets1[0].url, targets2[0].url);
    }
}
