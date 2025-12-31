use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Default timeout for all executors
    pub default_timeout: u64,

    /// Default retry count
    pub default_retry_count: u32,

    /// Directory containing external middleware executables
    pub external_middleware_dir: Option<String>,

    /// Pre-configured middleware chains for different modes
    pub middleware_chains: HashMap<String, Vec<MiddlewareConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    pub name: String,
    pub r#type: MiddlewareType,
    pub config: Option<serde_yaml::Value>,
    pub enabled: bool,
    pub order: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MiddlewareType {
    Builtin(String),
    External(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    pub default_headers: Option<HashMap<String, String>>,
    pub default_options: Option<HashMap<String, String>>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, anyhow::Error> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, anyhow::Error> {
        let config: Config = serde_yaml::from_str(yaml)?;
        Ok(config)
    }

    pub fn get_middleware_chain(&self, mode: &str) -> Vec<MiddlewareConfig> {
        self.middleware_chains
            .get(mode)
            .cloned()
            .unwrap_or_else(|| {
                // 默认中间件链
                let mut default_chain = Vec::new();

                default_chain.push(MiddlewareConfig {
                    name: "logger".to_string(),
                    r#type: MiddlewareType::Builtin("logger".to_string()),
                    config: None,
                    enabled: true,
                    order: Some(1),
                });

                default_chain
            })
    }
}