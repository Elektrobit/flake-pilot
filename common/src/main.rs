mod config;
use crate::config::cfgparse::FlakeCfgParser;
use std::path::PathBuf;

/// Example multiversion config usage
pub fn main() {
    for p in vec!["../../robot_tests/data/config-v1/firecracker.yaml", "../../robot_tests/data/config-v2/all-v2.yaml"] {
        let parser: FlakeCfgParser = FlakeCfgParser::new(PathBuf::from(p));
        let cfg: Option<config::itf::FlakeConfig> = parser.parse();
        if let Some(cfg) = cfg {
            println!("Version: {}", cfg.version());
        }
    }
}
