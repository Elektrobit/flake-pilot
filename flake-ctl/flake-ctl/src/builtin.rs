use std::{env::Args, fs, process::ExitCode};

use itertools::Itertools;

use crate::addons;

pub fn help(_args: Args) -> ExitCode {
    println!("-- Builtin --");
    [("help", "Print this list"), ("list", "List all installed flakes")]
        .into_iter()
        .for_each(|(name, desc)| println!("{name}\t\t{desc}"));
    println!("{}", addons::find_addons());
    ExitCode::SUCCESS
}

pub fn list() -> ExitCode {
    match fs::read_dir("/usr/share/flakes") {
        Ok(dir) => dir
            .filter_map(Result::ok)
            .filter_map(|x| x.path().file_stem().map(ToOwned::to_owned))
            .unique()
            .for_each(|x| println!("{}", x.to_str().unwrap_or_default())),
        Err(_) => return ExitCode::FAILURE,
    }

    ExitCode::SUCCESS
}
