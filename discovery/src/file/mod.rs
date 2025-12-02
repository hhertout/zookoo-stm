use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::Discovery;

#[cfg(test)]
mod tests;

/// File-based discovery that reads targets from a JSON file.
/// Generic over the target type, allowing type-safe discovery for different probe types.
#[derive(Clone)]
pub struct FileDiscovery<T: Clone + std::fmt::Debug + Send + Sync + 'static> {
    #[allow(dead_code)]
    file_path: PathBuf,
    targets: Arc<RwLock<Vec<T>>>,
}

impl<T> FileDiscovery<T>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    /// Create a new FileDiscovery instance and load targets synchronously
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        let file_path = file_path.into();
        let targets = Self::load_targets_sync(&file_path);
        Self { file_path, targets: Arc::new(RwLock::new(targets)) }
    }

    /// Load targets synchronously (used at startup)
    fn load_targets_sync(file_path: &PathBuf) -> Vec<T> {
        match fs::read_to_string(file_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(targets) => targets,
                Err(e) => {
                    log::error!("event=error msg=INVALID_CONFIGURATION");
                    log::error!(
                        "event=error msg=failed_to_parse_targets remediation=ignoring... path={} err={}",
                        file_path.display(),
                        e
                    );
                    panic!(
                        "Error while parsing the file... please ensure the file contains valid JSON targets"
                    );
                }
            },
            Err(e) => {
                log::error!("INVALID CONFIGURATION");
                log::error!("{}", e);
                panic!("Error while reading file... please check the file discovery configuration");
            }
        }
    }
}

impl<T> Discovery for FileDiscovery<T>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    type Target = T;

    fn discover(&self) -> Vec<Self::Target> {
        // Try non-blocking read, fallback to empty if locked
        self.targets.try_read().map(|guard| guard.clone()).unwrap_or_default()
    }

    fn update(&self) {
        // No update mechanism for file-based discovery
    }
}
