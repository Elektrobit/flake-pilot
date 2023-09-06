use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use std::path::Path;

// AppConfig represents application yaml configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub include: AppInclude,
    pub container: AppContainer,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppContainer {
    pub name: String,
    pub target_app_path: String,
    pub host_app_path: String,
    pub base_container: Option<String>,
    pub layers: Option<Vec<String>>,
    pub runtime: AppContainerRuntime,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppContainerRuntime {
    pub runas: Option<String>,
    pub resume: Option<bool>,
    pub attach: Option<bool>,
    pub podman: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppInclude {
    pub tar: Option<Vec<String>>,
}

impl AppConfig {
    pub fn from_file(config_file: &Path) -> Result<AppConfig> {
        let config = std::fs::File::open(config_file).context("Failed to open config")?;
        serde_yaml::from_reader(config).context("Failed to import config file")
    }
}
