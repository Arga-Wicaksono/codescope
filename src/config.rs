//! Configuration file management for codescope.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for codescope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub default_limit: Option<usize>,
    pub default_depth: Option<usize>,
    pub default_exclude: Option<Vec<String>>,
    pub default_extension: Option<String>,
    pub color: Option<bool>,
    pub web_timeout: Option<u64>,
    pub interactive: Option<bool>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_limit: Some(20),
            default_depth: None,
            default_exclude: None,
            default_extension: None,
            color: Some(true),
            web_timeout: Some(10),
            interactive: Some(false),
        }
    }
}

/// Get the default config file path (~/.codescope.json).
pub fn default_config_path() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CS_CONFIG") {
        return Some(PathBuf::from(path));
    }
    dirs::home_dir().map(|h| h.join(".codescope.json"))
}

/// Load configuration from file, falling back to defaults.
pub fn load_config() -> Config {
    let path = match default_config_path() {
        Some(p) => p,
        None => return Config::default(),
    };

    if !path.exists() {
        return Config::default();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(_) => Config::default(),
        },
        Err(_) => Config::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.default_limit, Some(20));
        assert_eq!(config.color, Some(true));
    }

    #[test]
    fn test_config_roundtrip() {
        let config = Config {
            default_limit: Some(50),
            color: Some(false),
            ..Config::default()
        };
        let json = serde_json::to_string(&config).unwrap();
        let loaded: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.default_limit, Some(50));
        assert_eq!(loaded.color, Some(false));
    }
}
