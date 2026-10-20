#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config could not be serialized: {0}")]
    Serialize(#[from] serde_json::Error),
}
