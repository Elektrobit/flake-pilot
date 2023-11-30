use core::panic;
use flakes::config::{cfgparse::FlakeCfgParser, itf::FlakeConfig};
use std::env;

/// Setup the test
fn setup(cfg_path: String) -> Option<FlakeConfig> {
    if let Ok(parser) = FlakeCfgParser::new(env::current_dir().unwrap().join("tests").join("data").join(&cfg_path), vec![]) {
        return parser.parse();
    }

    None
}

/// Run a test bundle
pub fn tb<T>(cfg_path: String, probe: T) -> ()
where
    T: Fn(Option<FlakeConfig>) -> () + panic::UnwindSafe,
{
    probe(setup(cfg_path));
}
