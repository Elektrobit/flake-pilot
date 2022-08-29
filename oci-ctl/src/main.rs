#[macro_use]
extern crate log;

use env_logger::Env;
use std::process::exit;

pub mod cli;
pub mod podman;
pub mod app;
pub mod deb;
pub mod app_config;
pub mod defaults;

fn main() {
    setup_logger();

    let args = cli::parse_args();

    match &args.command {
        // load
        cli::Commands::Load { oci } => {
            exit(podman::load(oci));
        },
        // list
        cli::Commands::List { } => {
            info!("Registered applications:");
            let app_names = app::app_names();
            if app_names.is_empty() {
                println!("No application(s) registered");
            } else {
                for app in app_names {
                    println!("- {}", app);
                }
            }
        },
        // register
        cli::Commands::Register { container, app, target } => {
            if app::init() {
                app::register(container, app, target.as_ref());
            }
        },
        // remove
        cli::Commands::Remove { container, app } => {
            if ! app.is_none() {
                app::remove(app.as_ref().map(String::as_str).unwrap());
            }
            if ! container.is_none() {
                app::purge(container.as_ref().map(String::as_str).unwrap());
            }
        }
        // build
        cli::Commands::BuildDeb { oci, app, repo, arch } => {
            exit(deb::ocideb(oci, repo, app, arch.as_ref()));
        }
    }
}

fn setup_logger() {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
