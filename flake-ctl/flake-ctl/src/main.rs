pub mod addons;
mod builtin;

use std::{
    env::Args,
    process::{Command, ExitCode},
};

use builtin::{help, list};
use clap::{arg, App, Arg};

fn main() -> ExitCode {
    let addons = addons::find_addons();

    let mut args =
        App::new("flake-ctl").next_help_heading("Builtin").arg(Arg::new("list").help(Some("List all registered flakes")));

    for (kind, list) in addons.addons.iter() {
        args = args.next_help_heading(kind.name());
        for addon in list {
            args = args.subcommand(App::new(addon.name.as_str()));
        }
    }

    match args.get_matches().subcommand() {
        Some(("list", _)) => list(),
        _ => ExitCode::FAILURE
            // "-V" => {println!("2.0.0"); ExitCode::SUCCESS},
            // name => external(name, args),
    }
}

fn external(name: &str, args: Args) -> ExitCode {
    let full_name = format!("flake-ctl-{name}");
    match Command::new(full_name).args(args).status() {
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
