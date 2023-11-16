use std::{
    ffi::OsStr,
    fs::{copy, create_dir_all},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Ok, Result};
use colored::Colorize;
use flakes::config::load_from_path;
use fs_extra::{copy_items, dir::CopyOptions};
use termion::clear;

pub(crate) fn build() -> Result<()> {
    let config = load_from_path(Path::new("src/flake"))?;
    let name = config.runtime().get_symlinks().unwrap().0.file_name().unwrap();

    print!("{}", " Setup... ".yellow().bold());
    setup(name).context("Setup failed")?;
    println!("{}\r{}", clear::CurrentLine, "Setup".green().bold());

    print!("{}", " Building Image... ".yellow().bold());
    let name = "image_name";
    let out = Command::new("src/build.sh").arg(name).stderr(Stdio::inherit()).output().context("Failed to run src/build.sh")?;
    if !out.status.success() {
        bail!("Failed to build image with build.sh")
    }
    println!("{}\r{} ({})", clear::CurrentLine, "Built Image".green().bold(), name);

    print!("{}", " Compiling... ".yellow().bold());
    Command::new("flake-ctl").arg("build").arg("compile").arg(".staging").arg("--target").arg("out").output()?;
    println!("{}\r{}", clear::CurrentLine, "Compiled".green().bold());

    println!("{} ({})", "Build finished".green().bold(), name);
    Ok(())
}

fn setup(name: &OsStr) -> Result<()> {
    create_dir_all("out").context("could not create output directory")?;

    let staging = Path::new(".staging/usr/share/flakes");
    copy("src/flake.yaml", staging.join(name).with_extension("yaml")).context("No flake.yaml")?;
    copy("src/options.yaml", ".flakes/package/options.yaml").context("No flake.yaml")?;
    copy_items(&["src/flake.d"], staging.join(name).with_extension("d"), &CopyOptions::default()).ok();
    Ok(())
}
