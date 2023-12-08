use std::{
    fs::{copy, create_dir_all},
    io::{stdout, Write},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Ok, Result};
use colored::Colorize;

use flakes::config;
use fs_extra::{copy_items, dir::CopyOptions};
use termion::clear;

use crate::{
    common::setup_flake,
    util::{check_build_sh, discover_project_root, project_name},
};

pub(crate) fn build(keep: bool) -> Result<()> {
    discover_project_root()?;

    let name = project_name()?;

    let conf = config::load_from_path(Path::new("src/flake")).context("Could not open flake configuration")?;
    let image_name = conf.runtime().image_name();

    print!("{}", " Building Image... ".yellow().bold());
    stdout().flush()?;
    check_build_sh()?;
    let out =
        Command::new("src/build.sh").arg(image_name).stderr(Stdio::inherit()).output().context("Failed to run src/build.sh")?;
    if !out.status.success() {
        bail!("Failed to build image with build.sh")
    }
    setup_flake(&name, image_name, false)?;
    println!("{}\r{} ({})", clear::CurrentLine, "Built Image".green().bold(), image_name);

    print!("{}", " Setup".yellow().bold());
    stdout().flush()?;
    setup(&name).context("Setup failed")?;
    println!("{}\r{}", clear::CurrentLine, "Setup".green().bold());

    print!("{}", " Compiling... ".yellow().bold());
    stdout().flush()?;
    Command::new("flake-ctl").args(["build", "compile", ".staging", "--target", "out"]).output()?;
    println!("{}\r{}", clear::CurrentLine, "Compiled".green().bold());
    
    if !keep {
        print!("{}", " Cleaning up... ".yellow().bold());
        stdout().flush()?;
        Command::new("podman").arg("rmi").arg(image_name).output()?;
        println!("{}\r{}", clear::CurrentLine, "Clean".green().bold());
    } else {
        println!("{}", "Skipping Cleanup".bright_black().bold());
    }

    println!("{}", "Build finished".green().bold());
    Ok(())
}

fn setup(name: &str) -> Result<()> {
    create_dir_all("out").context("could not create output directory")?;

    let staging = Path::new(".staging/usr/share/flakes");
    copy("src/flake.yaml", staging.join(name).with_extension("yaml")).context("No flake.yaml")?;
    print!(".");
    stdout().flush()?;
    copy("src/options.yaml", ".flakes/package/options.yaml").context("No flake.yaml")?;
    print!(".");
    stdout().flush()?;
    copy_items(&["src/flake.d"], staging.join(name).with_extension("d"), &CopyOptions::default()).ok();
    print!(".");
    stdout().flush()?;
    Ok(())
}
