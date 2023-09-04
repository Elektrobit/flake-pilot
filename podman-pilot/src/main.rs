#[macro_use]
extern crate log;

pub mod error;
#[cfg(test)]
pub mod tests;

use std::process::{ExitCode, Termination};

use config::config;
use env_logger::Env;
use error::FlakeError;

pub mod app_path;
pub mod config;
pub mod defaults;
pub mod podman;

fn main() -> ExitCode {
    setup_logger();
    // load config now so we can terminate early if the config is invalid
    config();
    // past here there should be no more panics

    let result = run();

    // TODO: implement cleanup function
    // cleanup()

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err}");
            err.report()
        }
    }
}

fn run() -> Result<(), FlakeError> {
    let program_path = app_path::program_abs_path();
    let program_name = app_path::basename(&program_path);

    let container = podman::create(&program_name)?;
    let cid = &container.0;
    podman::start(&program_name, cid)
}

fn setup_logger() {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "trace").write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
