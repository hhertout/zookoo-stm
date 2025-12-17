use async_trait::async_trait;
use configuration::model::discovery::DiscoveryFile;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::RwLock;
use tokio::sync::watch;

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
    version: Arc<AtomicU64>,
    update_tx: watch::Sender<u64>,
    update_rx: watch::Receiver<u64>,
}

impl<T> FileDiscovery<T>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    /// Create a new FileDiscovery instance and load targets synchronously
    /// Initialize targets as empty vec.
    pub fn new(conf: DiscoveryFile) -> Self {
        let (update_tx, update_rx) = watch::channel(0u64);
        // TODO: use conf object
        let file_path = PathBuf::from(conf.path);
        Self {
            file_path,
            targets: Arc::new(RwLock::new(vec![])),
            version: Arc::new(AtomicU64::new(0)),
            update_tx,
            update_rx,
        }
    }

    /// Load targets synchronously (used at startup)
    #[tracing::instrument(level = "debug", skip(self), fields(path = %self.file_path.display()))]
    fn load_targets_sync(&self) -> Vec<T> {
        match fs::read_to_string(&self.file_path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(targets) => targets,
                Err(e) => {
                    tracing::error!(
                        path = %self.file_path.display(),
                        error = %e,
                        "discovery_file_invalid_json"
                    );
                    panic!(
                        "Error while parsing the file... please ensure the file contains valid JSON targets"
                    );
                }
            },
            Err(e) => {
                tracing::error!(
                    path = %self.file_path.display(),
                    error = %e,
                    "discovery_file_read_failed"
                );
                panic!("Error while reading file... please check the file discovery configuration");
            }
        }
    }
}

#[async_trait]
impl<T> Discovery for FileDiscovery<T>
where
    T: Clone + std::fmt::Debug + Send + Sync + DeserializeOwned + 'static,
{
    type Target = T;

    #[tracing::instrument(
        level = "info",
        skip(self),
        fields(path = %self.file_path.display(), targets = tracing::field::Empty, version = tracing::field::Empty)
    )]
    async fn discover(&self) {
        let targets = self.load_targets_sync();
        tracing::Span::current().record("targets", targets.len());
        self.targets.write().await.clone_from(&targets);
        self.version.fetch_add(1, Ordering::Relaxed);
        tracing::Span::current().record("version", self.version());
        let _ = self.update_tx.send(self.version());
    }

    async fn get_targets(&self) -> Vec<Self::Target> {
        self.targets.read().await.clone()
    }

    fn version(&self) -> u64 {
        self.version.load(Ordering::Relaxed)
    }

    fn subscribe(&self) -> Option<watch::Receiver<u64>> {
        Some(self.update_rx.clone())
    }
}
