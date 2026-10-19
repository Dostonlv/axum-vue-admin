use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use validator::Validate;

// Config module entry
pub mod app;
pub mod auth;
pub mod database;
pub mod logging;
pub mod openapi;
pub mod redis;
pub mod smtp;
pub mod state;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub app_name: String,
    pub server: app::ServerConfig,
    pub database: database::DatabaseConfig,
    pub redis: redis::RedisConfig,
    pub log: logging::LogConfig,
    pub smtp: smtp::SmtpConfig
}

impl AppConfig {
    /// Load config from a file
    pub fn load_config<P: AsRef<Path>>(path: P) -> Result<AppConfig, anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let mut config: AppConfig = toml::from_str(&content)?;

        if let Ok(env_db_url) = std::env::var("DATABASE_URL") {
            config.database.url = env_db_url;
        }

        if let Ok(env_redis_url) = std::env::var("REDIS_URL") {
            config.redis.url = env_redis_url;
        }
        
        config.validate()?;
        Ok(config)
    }

    /// Write a default config file
    pub fn get_default_config() -> String {
        let default_config = Self::default();
        let toml_content = toml::to_string_pretty(&default_config);
        toml_content.unwrap_or_else(|_| "".to_string())
    }

    /// Validate the config
    pub fn validate(&self) -> Result<(), anyhow::Error> {
        self.server.validate()?;
        self.database.validate()?;
        self.redis.validate()?;
        self.log.validate()?;
        Ok(())
    }

    /// Get the server address
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_name: "Axum Vue Admin".to_string(),
            server: app::ServerConfig::default(),
            database: database::DatabaseConfig::default(),
            redis: redis::RedisConfig::default(),
            log: logging::LogConfig::default(),
            smtp: smtp::SmtpConfig::default(),
        }
    }
}