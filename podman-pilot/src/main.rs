#[macro_use]
extern crate log;
pub mod error;
use config::config;
use env_logger::Env;
use std::{
    env,
    path::Path,
    process::{ExitCode, Termination},
};
pub mod config;
pub mod defaults;
pub mod podman;

fn main() -> ExitCode {
    let env = Env::default().filter_or("MY_LOG_LEVEL", "trace").write_style_or("MY_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    config();

    let mut appdir = std::env::current_exe().unwrap();
    appdir.pop();
    appdir = appdir.join(Path::new(&env::args().next().unwrap().to_string()).file_name().unwrap());

    match podman::start(appdir) {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            error!("{err}");
            err.report()
        }
    }
}
