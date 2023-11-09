pub mod addons;
mod builtin;

use std::{
    env,
    process::{Command, ExitCode},
};

use builtin::list;
use clap::arg;
use colored::Colorize;

fn main() -> ExitCode {
    let addons = addons::find_addons();

    let mut args = clap::Command::new("flake-ctl")
        // .trailing_var_arg(true)
        .arg_required_else_help(true)
        .version("2.0.0")
        .about("Manage Flake Applications")
        .subcommand(clap::Command::new("list").about(Some("List all registered flakes")));

    let mut after_help = String::new();
    
    for (kind, list) in addons.addons.iter() {
        // TODO: Add kind base headings once clap has them
        // https://github.com/clap-rs/clap/issues/1553

        // Until then
        after_help.push_str(&format!("{}:\n", kind.name().underline().bold()));
        for addon in list {
            args = args.subcommand(
                clap::Command::new(clap::builder::Str::from(addon.name.clone()))
                .about(addon.description.as_ref().cloned().unwrap_or_default())
                .trailing_var_arg(true)
                .arg(arg!(<cmd> ... "args for the tool").required(false).allow_hyphen_values(true))
                // Remove this when subcommand headings work
                .hide(true),
            );
            after_help.push_str(&format!("  {: <16}{}\n", addon.name.bold(), addon.description.as_deref().unwrap_or_default()));
        }
        after_help.push('\n');
    }

    args = args.after_help(after_help);

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
