pub mod addons;
mod builtin;

use std::{
    env::Args,
    process::{Command, ExitCode},
};

use builtin::{help, list};

fn main() -> ExitCode {
    let mut args = std::env::args();
    args.next();

    match args.next() {
        None => help(args),
        Some(name) => match name.as_str() {
            "help" => help(args),
            "list" => list(),
            "-V" => {
                println!("2.0.0");
                ExitCode::SUCCESS
            }
            name => external(name, args),
        },
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
