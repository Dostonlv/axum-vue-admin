// # Logging config

use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Rotation {
    Hourly,
    Daily,
    Never,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LogConfig {
    pub level: String,
    pub file: String,
    pub rotation: LogRotationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LogRotationConfig {
    /// Max file size (MB)
    pub max_size: u64,
    /// Number of files to keep
    pub max_files: u32,
    /// Rotate by time
    pub daily: Rotation,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            level: "info".to_string(),
            file: "log.log".to_string(),
            rotation: LogRotationConfig::default(),
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        LogRotationConfig {
            max_size: 100,
            max_files: 7,
            daily: Rotation::Daily,
        }
    }
}
