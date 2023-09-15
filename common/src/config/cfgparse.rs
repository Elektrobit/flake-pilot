use serde::Deserialize;

use crate::config::cfg_v1::{self, FlakeCfgV1};

use super::itf::FlakeConfig;
use std::{
    fs::{self},
    io::Error,
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

        match self.get_version().as_str() {
            "1" => {
                parser = Box::new(FlakeCfgV1::new(self.root_path.to_owned()));
            }
            unsupported => {
                println!("ERROR: Unsupported configuration version: {}", unsupported);
                return None;
            }
        }

        Some(parser.parse())
    }
}
