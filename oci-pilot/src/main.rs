#[macro_use]
extern crate log;

#[cfg(test)]
pub mod tests;

use env_logger::Env;

pub mod app_path;
pub mod podman;
pub mod defaults;

fn main() {
    setup_logger();

    let program_path = app_path::program_abs_path();
    let program_name = app_path::basename(&program_path);

    podman::run(&program_name);
}

fn setup_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
