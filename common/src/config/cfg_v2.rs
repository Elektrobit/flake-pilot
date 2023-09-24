use crate::config::{cfgparse::FlakeCfgVersionParser, itf::FlakeConfig};
use serde::Deserialize;
use serde_yaml::Value;
use std::{collections::HashMap, io::Error};

#[derive(Deserialize, Debug)]
struct CfgV2Spec {
    pub(crate) version: u8,
    pub(crate) runtime: CfgV2Runtime,
    pub(crate) engine: CfgV2Engine,

    #[serde(rename = "static")]
    pub(crate) static_data: Option<Vec<String>>,
}

/// Runtime section
#[derive(Deserialize, Debug)]
struct CfgV2Runtime {
    pub(crate) name: String,
    pub(crate) path_map: HashMap<String, Value>,
    pub(crate) base_layer: Option<String>,
    pub(crate) layers: Option<Vec<String>>,
    pub(crate) user: Option<String>,
    pub(crate) instance: Option<String>,
}

///Engine section
#[derive(Deserialize, Debug)]
struct CfgV2Engine {
    pub(crate) pilot: String,
    pub(crate) args: Option<Vec<String>>,
    pub(crate) params: Option<Value>,
}

/// Configuration parser, v2.
pub struct FlakeCfgV2 {
    content: Value,
}

impl FlakeCfgV2 {
    pub fn new(content: Value) -> Self {
        FlakeCfgV2 { content }
    }

    fn as_cfg(&self, spec: CfgV2Spec) -> FlakeConfig {
        FlakeConfig { version: 2, ..Default::default() }
    }
}

impl FlakeCfgVersionParser for FlakeCfgV2 {
    fn parse(&self) -> Result<FlakeConfig, Error> {
        match serde_yaml::from_value::<CfgV2Spec>(self.content.to_owned()) {
            Ok(spec) => Ok(self.as_cfg(spec)),
            Err(err) => return Err(Error::new(std::io::ErrorKind::InvalidData, format!("Config v2 parse error: {}", err))),
        }
    }
}
