use std::{
    env::set_current_dir,
    fs::{self, copy, create_dir, create_dir_all, set_permissions, OpenOptions, Permissions},
    io::{stdout, Write},
    os::unix::fs::PermissionsExt,
};


use clap::Args;
use colored::Colorize;

use anyhow::{Context, Result};
use flake_ctl_build::{config::get_global, options::PackageOptionsBuilder};
use flakes::{
    config::load_from_path,
    paths::flake_dir_from,
    yamls::{make_template, string_to_template},
};
use fs_extra::{copy_items, dir::CopyOptions, file::read_to_string};
use termion::clear;

use crate::{util::{check_build_sh, project_name}, common::{setup_flake, image_name}};

const BUILD_SH: &str = "src/build.sh";

#[derive(Args)]
pub struct InitArgs {
    #[clap(flatten)]
    options: PackageOptionsBuilder,
}

pub fn new(name: String, i: InitArgs) -> Result<()> {
    create_dir(&name).context("Failed to create project directory")?;
    set_current_dir(&name).context("Failed to enter project directory")?;
    _init(&name, i)?;
    println!("{}\r{}", clear::CurrentLine, "Created project".green().bold());
    println!("Enter {} and run {} to package your flake", name.bold(), "flake-studio build".bold());
    Ok(())
}

pub fn init(init: InitArgs) -> Result<()> {
    let name = project_name()?;
    _init(&name, init)?;
    println!("{}\r{}", clear::CurrentLine, "Initialized".green().bold());
    println!("Run {} to package your flake", "flake-studio build".bold());
    Ok(())
}

fn _init(name: &str, init: InitArgs) -> Result<()> {
    let InitArgs { options } = init;
    let options = get_global().unwrap_or_default().update(options).fill_from_terminal();

    print!("{}", "Initializing".yellow().bold());
    structure().context("Failed to setup meta directories")?;
    print!(".");
    stdout().flush()?;
    options_file(options).context("Failed to create options.yaml")?;
    print!(".");
    stdout().flush()?;
    setup_flake(name, &image_name(name), true).context("Failed to init flake")?;
    print!(".");
    stdout().flush()?;
    yaml(name).context("Failed to create config files")?;
    print!(".");
    stdout().flush()?;
    build_sh().context("Failed to create build script")?;
    print!(".");
    stdout().flush()?;
    Ok(())
}

fn structure() -> Result<()> {
    create_dir_all(".flakes/package")?;
    create_dir_all(".flakes/studio")?;
    create_dir_all("src")?;
    create_dir_all(".staging")?;
    create_dir_all("out")?;
    Ok(())
}

fn options_file(options: PackageOptionsBuilder) -> Result<()> {
    let config: String = serde_yaml::to_string(&options)?;
    OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open("src/options.yaml")?
        .write_all(make_template(&options, Default::default())?.as_bytes())?;
    OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(".flakes/package/options.yaml")?
        .write_all(config.as_bytes())?;
    Ok(())
}

fn yaml(app: &str) -> Result<()> {
    let staging = flake_dir_from(Some(".staging"));
    let config = load_from_path(&staging.join(app))?;

    let name = config.runtime().get_symlinks().unwrap().0.file_name().unwrap();

    let yaml_content = read_to_string(staging.join(name).with_extension("yaml"))?;
    let doc = Default::default();
    let yaml_content = string_to_template(yaml_content, doc);
    OpenOptions::new().create(true).truncate(true).write(true).open("src/flake.yaml")?.write_all(yaml_content.as_bytes())?;

    copy_items(&[staging.join(name).with_extension("d")], "src/flake.d", &CopyOptions::default()).ok();
    Ok(())
}

fn build_sh() -> Result<()> {
    if home::home_dir().map(|dir| dir.join(".flakes/package/build.sh")).and_then(|dir| copy(dir, BUILD_SH).ok()).is_none() {
        fs::write(BUILD_SH, include_str!("../default_build_sh"))?;
    }
    check_build_sh()?;
    set_permissions(BUILD_SH, Permissions::from_mode(0o777))?;
    Ok(())
}
