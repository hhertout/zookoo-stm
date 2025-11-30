pub mod file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryType {
    File,
    Api,
}

pub trait Discovery {
    /// Target associated type (e.g. a struct describing an HTTP target or an ICMP target)
    type Target: Clone + std::fmt::Debug + Send + Sync + 'static;

    /// Return current targets. The Discovery is already specialized for a specific probe type.
    fn discover(&self) -> Vec<Self::Target>;

    /// Refresh discovery state (async operation).
    fn update(&self);
}
