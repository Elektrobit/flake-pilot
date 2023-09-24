use crate::config::{cfgparse::FlakeCfgVersionParser, itf::FlakeConfig};
use serde_yaml::Value;
use std::io::Error;

/// Configuration parser, v2.
pub struct FlakeCfgV2 {
    #[allow(dead_code)]
    content: Value,
}

impl FlakeCfgV2 {
    pub fn new(content: Value) -> Self {
        FlakeCfgV2 { content }
    }
}

impl FlakeCfgVersionParser for FlakeCfgV2 {
    fn parse(&self) -> Result<FlakeConfig, Error> {
        Ok(FlakeConfig::new(Some(2)))
    }
}
