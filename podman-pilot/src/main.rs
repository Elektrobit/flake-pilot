#[macro_use]
extern crate log;

pub mod error;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{ExitCode, Termination},
};

use config::config;
use env_logger::Env;
use error::FlakeError;

pub mod config;
pub mod defaults;
pub mod podman;

fn setup_logger() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "trace").write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);
}

fn run() -> Result<(), FlakeError> {
    let program_name = Path::new(fs::canonicalize(PathBuf::from(env::args().next().unwrap())).unwrap().to_str().unwrap())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    podman::start(&program_name, &podman::create(&program_name)?.0)
}

fn main() -> ExitCode {
    setup_logger();
    config();

    match run() {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err}");
            err.report()
        }
    }
}
