use std::{fs, path::Path, env::{set_current_dir, current_dir}};

use anyhow::{Context, Result, bail};

use colored::Colorize;

pub fn discover_project_root() -> Result<()> {
    let p = Path::new(".flakes/studio");
    while !p.exists() {
        if set_current_dir("..").is_err() || current_dir()? == Path::new("/") {
            bail!("You are not currently inside of a flake-studio project")
        }
    }
    Ok(())
}

pub fn check_build_sh() -> Result<()> {
    let content = fs::read_to_string("src/build.sh").context("Could not open 'build.sh'")?;
    if !content.starts_with("#!") {
        println!(
            "{} {}",
            "WARNING".bold().yellow(),
            "'build.sh' does not contain a shebang (#!/bin/sh) and might not be executed correctly".yellow()
        )
    }

    Ok(())
}

pub fn project_name() -> Result<String> {
    let cur_dir = current_dir()?;
    Ok(cur_dir.file_name().unwrap().to_string_lossy().to_string())
}

