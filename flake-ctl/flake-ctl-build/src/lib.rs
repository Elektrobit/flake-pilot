use std::{
    path::{Path, PathBuf},
    process::{Command, exit}, env,
};

pub mod config;
pub mod options;

use anyhow::{anyhow, bail, Context, Result};
use clap::{arg, ArgAction, ArgMatches, Args};
use flakes::{
    config::{itf::FlakeConfig, load_from_target, FLAKE_DIR},
    paths::PathExt,
};
use fs_extra::{
    copy_items, dir,
    file::{self, copy},
};
use options::{determine_options, PackageOptions, PackageOptionsBuilder};
use tempfile::tempdir;

pub fn run<B: Builder + Args>() -> Result<()> {
    let matches = B::augment_args(cli()).get_matches();
    let builder = B::from_arg_matches(&matches).context("Failed to extract builder specific args")?;

    let fake_root_temp = tempdir()?;

    let fake_root = fake_root_temp.path();

    let target_app = matches.get_one::<String>("app").ok_or(anyhow!("No target app provided"))?;
    let target_app = Path::new(target_app);
    let flake_name = target_app.file_name().unwrap().to_string_lossy();
    let flake_name = flake_name.as_ref();

    let temp_location = tempdir()?;
    let cli_location = matches.get_one::<String>("location");
    let cli_location = cli_location.cloned().map(PathBuf::from);
    let location = cli_location.as_deref().unwrap_or(temp_location.path());

    let options = determine_options(&matches)?;

    if !matches.get_flag("compile") {
        if let Some(image_name) = find_image_name(&matches) {
            std::process::Command::new("flake-ctl")
                .arg("podman")
                .arg("register")
                .arg("--root")
                .arg(fake_root)
                .arg("--app")
                .arg(target_app)
                .arg("--container")
                .arg(image_name)
                .args(matches.get_many::<String>("trailing").unwrap_or_default())
                .status()
                .context("Failed to register temporary flake")?;
        } else {
            bail!("Could not determine container name, maybe try passing --container")
        }

        let info = SetupInfo {
            config: load_from_target(Some(fake_root), Path::new(flake_name))?,
            flake_name: flake_name.to_owned(),
            options: options.clone(),
        };

        let BuilderInfo { image_location, config_location } = builder.setup(location, info)?;

        if !matches.get_flag("no-image") {
            if let Some(archive) = matches.get_one::<String>("from-tar") {
                copy(archive, image_location.join(&options.name), &file::CopyOptions::new()).context("Failed to copy archive")?;
            } else {
                export_flake(flake_name, fake_root, &image_location).context("Failed to export flake")?;
            }
        }
        let cfg_location = fake_root.join_ignore_abs(FLAKE_DIR.as_path()).join(flake_name);
        copy_configs(&cfg_location, &config_location).context("Failed to copy configs")?;
    }

    if !matches.get_flag("dry-run") {
        let info = CompileInfo {
            config: load_from_target(Some(location), Path::new(flake_name))?,
            options,
            flake_name: flake_name.to_owned(),
        };
        builder.compile(location, info, Path::new(&matches.get_one::<String>("output").cloned().unwrap_or(String::from("."))))
            .context("Failed to compile flake")?;
    }

    Ok(())
}

fn cli() -> clap::Command {
    let cmd = clap::Command::new("flake-ctl-build")
        .arg(arg!(-a --app <APP> "Name of the app on the host"))
        .arg(arg!(-c <OCI> "Build from the given oci container").id("from-oci").long("from-oci"))
        .arg(arg!(-t <TAR> "Build from tarball. NOTE: file should have extension \".tar.gz\"").id("from-tar").long("from-tar"))
        .arg(arg!(-o --output <OUTPUT> "Output directory or filename (default: working dir, name depends on package manager)"))
        .arg(arg!(-l --location <LOCATION> "Where to build the package (default: tempdir)"))
        .arg(arg!("Only assemble bundle, do not compile").id("dry-run").long("dry-run").action(ArgAction::SetTrue))
        .arg(arg!(--compile "Compile a pre-existing bundle given by 'location'"))
        .arg(arg!("Do not include an image in this flake").id("no-image").long("no-image").action(ArgAction::SetTrue))
        .arg(arg!(--ci "Skip all potential user input"))
        .arg(arg!(--image <IMAGE> "Override the image used in the flake (otherwise inferred)"))
        .arg(arg!(trailing: ... "Trailing args will be forwarded to the flake config").trailing_var_arg(true).hide(true));
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

    copy_items(&configs, bundling_dir, &dir::CopyOptions::new().overwrite(true))?;
    Ok(())
}

pub struct SetupInfo {
    pub config: FlakeConfig,
    pub options: PackageOptions,
    pub flake_name: String,
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
    fn setup(&self, location: &Path, info: SetupInfo) -> Result<BuilderInfo>;
    fn compile(&self, location: &Path, info: CompileInfo, target: &Path) -> Result<()>;
}

fn find_image_name(matches: &ArgMatches) -> Option<String> {
    if let Some(container) = matches.get_one::<String>("image") {
        Some(container).cloned()
    } else if let Some(container) = matches.get_one::<String>("from-oci") {
        Some(container).cloned()
    } else if let Some(tar) = matches.get_one::<String>("from-tar") {
        Path::new(tar).file_name()?.to_string_lossy().strip_suffix(".tar.gz").map(str::to_owned)
    } else {
        None
    }
}

pub fn about(about: &str) {
    if env::args().nth(1).as_deref() == Some("about") {
        println!("{about};Packager");
        exit(0);
    }
}
