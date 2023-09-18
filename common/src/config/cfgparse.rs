use super::{cfg_v2::FlakeCfgV2, itf::FlakeConfig};
use crate::config::cfg_v1::FlakeCfgV1;
use serde::Deserialize;
use serde_yaml::Value;
use std::{
    fs::{self},
    io::Error,
    path::PathBuf,
};

#[derive(Deserialize, Debug)]
struct ConfigVersion {
    version: Option<u8>,
}

pub trait FlakeCfgVersionParser {
    /// Parse the configuration
    /// Accepts _root path_ to where the configuration is installed.
    /// It assumes `.d` directory as a subdirectory, i.e. `ROOT_PATH/<flake>.d`
    /// directory to overlay.
    fn parse(&self) -> FlakeConfig;
}

pub struct FlakeCfgParser {
    cfg_path: PathBuf,
    cfg_d_paths: Vec<PathBuf>,
}

impl FlakeCfgParser {
    pub fn new(cfg_path: PathBuf, cfg_d_paths: Vec<PathBuf>) -> Result<Self, Error> {
        for p in vec![&cfg_path].into_iter().chain(&cfg_d_paths) {
            if !p.exists() {
                return Err(Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Configuration file {} was not found", p.to_str().unwrap()),
                ));
            }
        }

        Ok(FlakeCfgParser { cfg_path, cfg_d_paths })
    }

    /// Merge YAML config source
    fn merge_values(base: Value, update: Value) -> Value {
        match (base, update) {
            (Value::Mapping(mut base), Value::Mapping(update)) => {
                // TODO: This could be written nicer by somebody who wants to fight with the lifetimes of `Mapping::entry`
                for (key, value) in update {
                    let old = base.get(&key).cloned().unwrap_or_default();
                    base.insert(key, Self::merge_values(old, value));
                }
                base.into()
            }
            (base, Value::Null) => base,
            (_, update) => update,
        }
    }

    /// Get the configuration version from the base config (explicitly ignoring the .d part)
    fn get_version(&self) -> u8 {
        if let Ok(data) = &fs::read_to_string(self.cfg_path.to_owned()) {
            let cfg_version: ConfigVersion = serde_yaml::from_str::<ConfigVersion>(data).unwrap();
            if let Some(version) = cfg_version.version {
                return version;
            }
        }

        1
    }

    /// Parse given config
    pub fn parse(&self) -> Option<FlakeConfig> {
        let parser: Box<dyn FlakeCfgVersionParser> = match self.get_version() {
            1 => Box::new(FlakeCfgV1::new(self.cfg_path.to_owned())), // XXX: prbably should take serde_yaml::Value here already
            2 => Box::new(FlakeCfgV2::new(self.cfg_path.to_owned())), // XXX: prbably should take serde_yaml::Value here already
            unsupported => {
                println!("ERROR: Unsupported configuration version: {}", unsupported);
                return None;
            }
        };

        Some(parser.parse())
    }
}
