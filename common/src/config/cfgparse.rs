use super::{cfg_v2::FlakeCfgV2, itf::FlakeConfig};
use crate::config::cfg_v1::FlakeCfgV1;
use serde::Deserialize;
use std::{
    fs::{self},
    path::PathBuf,
};

#[derive(Deserialize, Debug)]
struct ConfigVersion {
    version: Option<String>,
}

pub trait FlakeCfgVersionParser {
    /// Parse the configuration
    /// Accepts _root path_ to where the configuration is installed.
    /// It assumes `.d` directory as a subdirectory, i.e. `ROOT_PATH/<flake>.d`
    /// directory to overlay.
    fn parse(&self) -> FlakeConfig;
}

pub struct FlakeCfgParser {
    root_path: PathBuf,
}

impl FlakeCfgParser {
    pub fn new(path: PathBuf) -> Self {
        FlakeCfgParser { root_path: path }
    }

    /// Get the configuration version
    fn get_version(&self) -> String {
        if !self.root_path.exists() {
            return "-1".to_string();
        }

        if let Ok(data) = &fs::read_to_string(self.root_path.to_str().unwrap()) {
            let cfg_version: ConfigVersion = serde_yaml::from_str::<ConfigVersion>(data).unwrap();
            if let Some(version) = cfg_version.version {
                return version;
            }
        }

        "1".to_string()
    }

    /// Parse given config
    pub fn parse(&self) -> Option<FlakeConfig> {
        let parser: Box<dyn FlakeCfgVersionParser>;
        println!(">>>>>>>>>>>>>>>>>>? {}", self.get_version());

        match self.get_version().as_str() {
            "1" => {
                parser = Box::new(FlakeCfgV1::new(self.root_path.to_owned()));
            }
            "2" => {
                parser = Box::new(FlakeCfgV2::new(self.root_path.to_owned()));
            }
            "-1" => {
                println!("ERROR: configuration file was not found");
                return None;
            }
            unsupported => {
                println!("ERROR: Unsupported configuration version: {}", unsupported);
                return None;
            }
        }

        Some(parser.parse())
    }
}
