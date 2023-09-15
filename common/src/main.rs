mod config;
use crate::config::cfgparse::FlakeCfgParser;
use std::path::PathBuf;

/// Example multiversion config usage
pub fn main() {
    println!("{:-<80}", "");

    let parser: FlakeCfgParser = FlakeCfgParser::new(PathBuf::from("../../robot_tests/data/config-v1/firecracker.yaml"));
    let cfg: Option<config::itf::FlakeConfig> = parser.parse();
    if let Some(cfg) = cfg {
        println!("{:?}", cfg);
    }
}
