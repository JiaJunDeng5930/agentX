use std::time::Duration;

/// Storage policy for conversation/rollout data and upstream model storage.
///
/// - `Full`: store full content locally; allow upstream store flag.
/// - `HeadersOnly`: store minimal metadata; upstream store disabled.
/// - `None`: do not store content locally; upstream store disabled.
/// - `Ttl(d)`: store full content locally but eligible for later down‑tiering;
///             upstream store enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoragePolicy {
    Full,
    HeadersOnly,
    None,
    Ttl(Duration),
}

impl StoragePolicy {
    /// Whether the upstream model API should be asked to store responses.
    /// Maps to the `store` field in Responses API requests.
    pub fn upstream_store_enabled(&self) -> bool {
        match self {
            StoragePolicy::Full => true,
            StoragePolicy::HeadersOnly => false,
            StoragePolicy::None => false,
            StoragePolicy::Ttl(_) => true,
        }
    }
}
