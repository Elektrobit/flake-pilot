use std::{fs, process::ExitCode};

use itertools::Itertools;

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
