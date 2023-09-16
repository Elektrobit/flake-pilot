use crate::config::{cfgparse::FlakeCfgVersionParser, itf::FlakeConfig};
use std::path::PathBuf;

/// Configuration parser, v1.
///
/// It is the original version of the config,
/// prior to 2.2.19 Flakes version.
///
pub struct FlakeCfgV1 {
    path: PathBuf,
}

impl FlakeCfgV1 {
    pub fn new(path: PathBuf) -> Self {
        FlakeCfgV1 { path }
    }
}

impl FlakeCfgVersionParser for FlakeCfgV1 {
    fn parse(&self) -> super::itf::FlakeConfig {
        println!("Looking for {}", self.path.to_str().unwrap());
        FlakeConfig::default()
    }
}
