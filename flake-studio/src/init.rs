use std::{
    fs::{copy, create_dir_all, OpenOptions},
    io::Write,
    path::Path,
    process::Command,
};

use colored::Colorize;

use anyhow::{bail, Context, Result};
use flake_ctl_build::{config::get_global, PackageOptionsBuilder};
use flakes::{config::load_from_path, paths::flake_dir_from};
use fs_extra::{copy_items, dir::CopyOptions};
use termion::clear;

pub fn init(options: PackageOptionsBuilder, scratch: bool, app: &Path) -> Result<()> {
    new(options.fill_from_terminal(), scratch, app)
}

pub fn new(options: PackageOptionsBuilder, scratch: bool, app: &Path) -> Result<()> {
    eprint!("{}", "Initializing".yellow().bold());
    structure().context("Failed to setup meta directories")?;
    eprint!(".");
    options_file(options, scratch).context("Failed to create options.yaml")?;
    eprint!(".");
    gitignore().context("Failed to create .gitignore")?;
    eprint!(".");
    flake(app).context("Failed to init flake")?;
    eprint!(".");
    yaml(app).context("Failed to create config files")?;
    eprint!(".");
    eprintln!("{}\r{}", clear::CurrentLine, "Initialized".green().bold());
    eprintln!("Run {} to package your flake", "flake-studio build".bold());
    Ok(())
}

fn structure() -> Result<()> {
    create_dir_all(".flakes/package")?;
    create_dir_all("src")?;
    create_dir_all(".staging")?;
    create_dir_all("out")?;
    Ok(())
}

fn flake(app: &Path) -> Result<()> {
    let out = Command::new("flake-ctl-build")
        .args(["image", "podman", "ubuntu"])
        .arg(app)
        .args(["--location", ".staging", "--ci", "--keep", "--dry-run"])
        .output()?;
    if !out.status.success() {
        let x = String::from_utf8_lossy(&out.stderr);
        bail!("{}", x)
    }
    Ok(())
}

fn options_file(options: PackageOptionsBuilder, scratch: bool) -> Result<()> {
    let options = if !scratch {
        if let Ok(global) = get_global() {
            global.update(options)
        } else {
            options
        }
    } else {
        options
    };
    let config: String = serde_yaml::to_string(&options)?;
    OpenOptions::new().truncate(true).create(true).write(true).open("src/options.yaml")?.write_all(config.as_bytes())?;
    Ok(())
}

fn yaml(app: &Path) -> Result<()> {
    let staging = flake_dir_from(Some(".staging"));
    let config = load_from_path(&staging.join(app.file_name().unwrap()))?;

    let name = config.runtime().get_symlinks().unwrap().0.file_name().unwrap();
    copy(staging.join(name).with_extension("yaml"), "src/flake.yaml").context("No flake.yaml")?;
    copy_items(&[staging.join(name).with_extension("d")], "src/flake.d", &CopyOptions::default()).ok();
    Ok(())
}

fn gitignore() -> Result<()> {
    OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(".flakes/package/.gitignore")?
        .write_all("out\n.staging\n".as_bytes())?;
    Ok(())
}
