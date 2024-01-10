use std::{
    path::{Path, PathBuf},
    process::Command,
};

mod config;
pub mod options;

use anyhow::{anyhow, bail, Context, Result};
use clap::{arg, Args};
use flakes::{
    config::{itf::FlakeConfig, load_from_path, load_from_target, FLAKE_DIR},
    paths::{PathExt, RootedPath},
};
use fs_extra::{
    copy_items, dir,
    file::{self, copy},
};
use options::{PackageOptions, PackageOptionsBuilder};
use tempfile::tempdir;

pub fn run<B: Builder>() -> Result<()> {
    let matches = cli().get_matches();

    let fake_root_temp = tempdir()?;

    let fake_root = fake_root_temp.path();

    let flake_name = matches.get_one::<String>("target_app").ok_or(anyhow!("No target app provided"))?;

    let temp_location = tempdir()?;
    let cli_location = matches.get_one("location").cloned();
    let location = cli_location.unwrap_or(temp_location.path());

    let options = PackageOptionsBuilder::default().fill_from_terminal().build()?;

    if !matches.get_flag("compile") {
        if let Some(image_name) = matches.get_one::<String>("from-oci") {
            std::process::Command::new("flake-ctl")
                .arg("podman")
                .arg("register")
                .arg("--root")
                .arg(fake_root)
                .arg("--app")
                .arg(flake_name)
                .arg("--container")
                .arg(image_name)
                .args(matches.get_many::<String>("trailing").unwrap_or_default())
                .status()
                .context("Failed to register temporary flake")?;
        }

        let info = SetupInfo { config: load_from_target(Some(fake_root), Path::new(flake_name))?, options: options.clone() };

        let BuilderInfo { image_location, config_location } = B::setup(location, info)?;

        if let Some(archive) = matches.get_one::<String>("from-tar") {
            copy(archive, image_location, &file::CopyOptions::new()).context("Failed to copy archive")?;
        } else {
            export_flake(flake_name, fake_root, &image_location).context("Failed to export flake")?;
        }
        let cfg_location = fake_root.join_ignore_abs(FLAKE_DIR.as_path()).join(flake_name);
        copy_configs(&cfg_location, &config_location).context("Failed to copy configs")?;
    }

    if !matches.get_flag("dry-run") {
        let info = CompileInfo {
            config: load_from_target(Some(fake_root), Path::new(flake_name))?,
            options,
            flake_name: flake_name.clone(),
        };
        B::compile(location, info, matches.get_one("output").cloned().unwrap_or(Path::new(".")))
            .context("Failed to compile flake")?;
    }

    Ok(())
}

fn cli() -> clap::Command {
    let cmd = clap::Command::new("flake-ctl-build")
        .about("")
        .arg(arg!(-a --target-app <APP> "Name of the app on the host"))
        .arg(arg!(-c --from-oci <OCI> "Build from the given oci container"))
        .arg(arg!(-t --from-tar <TAR> "Build from the given .tar.gz archive"))
        .arg(arg!(-o --output <OUTPUT> "Output directory or filename (default: working dir, name depends on package manager)"))
        .arg(arg!(-l --location <LOCATION> "Where to build the package (default: tempdir)"))
        .arg(arg!(--dry-run "Only assemble bundle, do not compile"))
        .arg(arg!(--compile "Compile a pre-existing bundle"))
        .arg(arg!(trailing: ... "Trailing args will be forwarded to the flake config").trailing_var_arg(true));
    PackageOptionsBuilder::augment_args(cmd)
}

pub fn export_flake(name: &str, root: &Path, target: &Path) -> Result<()> {
    let config = load_from_target(Some(root), Path::new(name))?;

    let out = Command::new("flake-ctl")
        .arg(config.engine().pilot())
        .arg("export")
        .arg("--root")
        .arg(root)
        .arg(name)
        .arg(target)
        .output()?;

    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr))
    }
    Ok(())
}

pub fn copy_configs(path: &Path, bundling_dir: &Path) -> Result<()> {
    let configs = [path.with_extension("yaml"), path.with_extension("d")];

    copy_items(&configs, bundling_dir, &dir::CopyOptions::new())?;
    Ok(())
}

pub struct SetupInfo {
    pub config: FlakeConfig,
    pub options: PackageOptions,
}

pub struct CompileInfo {
    pub config: FlakeConfig,
    pub options: PackageOptions,
    pub flake_name: String,
}

pub struct BuilderInfo {
    pub image_location: PathBuf,
    pub config_location: PathBuf,
}

pub trait Builder {
    fn setup(location: &Path, info: SetupInfo) -> Result<BuilderInfo>;
    fn compile(location: &Path, info: CompileInfo, target: &Path) -> Result<()>;
}
