use crate::config::{cfgparse::FlakeCfgVersionParser, itf::FlakeConfig};
use std::path::PathBuf;

/// Configuration parser, v2.
pub struct FlakeCfgV2 {
    path: PathBuf,
}

impl FlakeCfgV2 {
    pub fn new(path: PathBuf) -> Self {
        FlakeCfgV2 { path }
    }
}

impl FlakeCfgVersionParser for FlakeCfgV2 {
    fn parse(&self) -> super::itf::FlakeConfig {
        println!("Version 2");
        println!("Looking for {}", self.path.to_str().unwrap());
        FlakeConfig::default()
    }
}
