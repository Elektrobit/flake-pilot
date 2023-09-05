use std::{
    env::{self, split_paths, Args},
    fs,
    process::ExitCode,
};

use itertools::Itertools;

pub fn help(_args: Args) -> ExitCode {
    println!("-- Builtin --");
    [("help", "print this list"), ("list", "list all installed flakes")]
        .into_iter()
        .for_each(|(name, desc)| println!("{name}\t\t{desc}"));
    println!("-- Addons --\n{}", find_addons().join("\n"));
    ExitCode::SUCCESS
}

fn find_addons() -> Vec<String> {
    split_paths(&env::var_os("PATH").unwrap_or_default())
        .map(fs::read_dir)
        .filter_map(Result::ok)
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|x| x.file_name().to_str().map(str::to_owned))
        .filter_map(|name| name.strip_prefix("flake-ctl-").map(str::to_owned))
        .unique()
        .collect()
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
