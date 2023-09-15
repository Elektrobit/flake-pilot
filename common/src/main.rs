mod config;
use crate::config::cfgparse::FlakeCfgParser;
use std::path::PathBuf;

pub fn main() {
    println!("{:-<80}", "");
    let x = FlakeCfgParser::new(PathBuf::from("../../robot_tests/data/config-v1/firecracker.yaml"));
    let c = x.parse();
}
