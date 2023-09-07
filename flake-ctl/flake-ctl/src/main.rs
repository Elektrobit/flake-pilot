pub mod addons;
mod builtin;

use std::{
    env,
    process::{Command, ExitCode},
};

use builtin::list;
use clap::{arg, App};

fn main() -> ExitCode {
    let addons = addons::find_addons();

    let mut args = App::new("flake-ctl")
        // .trailing_var_arg(true)
        .arg_required_else_help(true)
        .version("2.0.0")
        .about("Manage Flake Applications")
        .subcommand(App::new("list").about(Some("List all registered flakes")));

    for (_kind, list) in addons.addons.iter() {
        // TODO: Needs clap 4.x.x to properly work
        // args = args.subcommand_help_heading(kind.name());
        for addon in list {
            args = args.subcommand(
                App::new(addon.name.as_str())
                    .about(addon.description.as_deref())
                    .trailing_var_arg(true)
                    .arg(arg!(<cmd> ... "args for the tool").required(false)),
            );
        }
    }

    match args.get_matches().subcommand() {
        Some(("list", _)) => list(),
        Some((name, _)) => external(name),
        _ => ExitCode::FAILURE,
    }
}

fn external(name: &str) -> ExitCode {
    let full_name = format!("flake-ctl-{name}");
    match Command::new(full_name).args(env::args().skip(2)).status() {
        Ok(output) => (output.code().unwrap_or_default() as u8).into(),
        Err(error) => {
            match error.kind() {
                std::io::ErrorKind::PermissionDenied => {
                    eprintln!("You do not have permission to run \"flake-ctl {name}\", maybe try sudo.")
                }
                std::io::ErrorKind::NotFound => {
                    eprintln!("Unknown sub command \"{name}\". See `flake-ctl help` for a list of available commands.")
                }
                _ => eprintln!("Could not run \"flake-ctl-{name}\". Underlying cause: {error}"),
            };
            ExitCode::FAILURE
        }
    }
}
