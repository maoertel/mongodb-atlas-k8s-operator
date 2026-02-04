use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

/// Configuration for the operator
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub atlas_user: AtlasUserConfig,
}

/// Configuration specific to AtlasUser reconciliation
#[derive(Debug, Clone, Deserialize)]
pub struct AtlasUserConfig {
    /// How long to wait before requeuing a reconciliation
    #[serde(with = "humantime_serde")]
    pub requeue_duration: Duration,
    /// Whether it's safe to delete users from Atlas when the K8s resource is deleted
    pub safe_to_delete: bool,
}

impl Default for AtlasUserConfig {
    fn default() -> Self {
        Self {
            requeue_duration: Duration::from_secs(60),
            safe_to_delete: false,
        }
    }
}

impl Config {
    /// Loads configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_yaml::from_str(&content)?;
        Ok(config)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to parse config file: {0}")]
    Parse(#[from] serde_yaml::Error),
}
