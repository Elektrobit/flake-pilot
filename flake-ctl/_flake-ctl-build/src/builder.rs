use std::{
    fs::{create_dir_all, remove_dir_all},
    path::{Path, PathBuf},
    process::Command,
};

use crate::options::{PackageOptions, PackageOptionsBuilder};
use anyhow::{anyhow, bail, Context, Ok, Result};
use clap::{Args, FromArgMatches, Parser, Subcommand};
use flakes::{
    config::{self, itf::FlakeConfig, FLAKE_DIR},
    paths::{PathExt, RootedPath},
};
use fs_extra::{copy_items, dir::CopyOptions};
use tempfile::tempdir;

pub trait FlakeBuilder {
    /// Brief info about the package manager used by this builder
    fn description(&self) -> &str;

    /// Create the infrastructure to build the flake package in the given directory
    fn setup(&self, location: &Path) -> Result<()>;

    /// Copy all the necessary files to the build location and create the configuration for the package (e.g. a .spec file)
    fn create_bundle(
        &self, flake_path: &RootedPath, args: &BuilderArgs, options: &PackageOptions, config: &FlakeConfig, location: &Path,
    ) -> Result<()>;

    /// Perform the actual build process.
    ///
    /// If `target` is given store the resulting package there, otherwise put it into the current working directory
    fn build(&self, options: &PackageOptions, target: Option<&Path>, location: &Path) -> Result<()>;

    /// Remove all the temporary infrastructure created in [setup]
    ///
    /// This may be run before _and_ after each build
    ///
    /// Only modify the contents of `location`, if the directory itself needs to be deleted this will be handled by [cleanup_default_directory]
    fn purge(&self, location: &Path) -> Result<()>;

    /// What directory to use when no build location is given, this defaults to a new tmp directory
    ///
    /// Override this for your builder if there is another sensible default
    ///
    /// Keep in mind that you may also have to adjust [cleanup_default_directory]
    fn get_default_build_directory(&self) -> Result<PathBuf> {
        Ok(tempdir()?.into_path())
    }

    /// Cleanup the directory structure of the default directory, this is only called when no build location was given
    ///
    /// Default behaviour is to just delete the directory (assumed to be a tmp directory created by [get_default_build_directory]),
    /// override this function if this is undesired
    fn cleanup_default_directory(&self, dir: &Path) -> Result<()> {
        Ok(remove_dir_all(dir)?)
    }

    /// Run the full packaging.
    ///
    /// If `target` is given store the resulting package there, otherwise put it into the current working directory
    /// If `location` is given the package is build there, otherwise it is constructed in a tmp directory
    ///
    /// [build] is only run if `build` is `true`
    ///
    /// [cleanup] is not run if `keep` is `true`
    fn execute(&self, options: PackageOptions, mode: Mode) -> Result<()> {
        let args = mode.args();

        let cleanup_default = args.location.is_none();

        let location = match args.location.as_ref() {
            Some(location) => location.to_owned(),
            None => self.get_default_build_directory()?,
        };

        let flake_path = mode.prepare()?;
        let flake_name = flake_path.path().file_name().ok_or_else(|| anyhow!("Invalid flake path"))?;

        // Clean the location if it is reused, ignore errors here
        self.purge(&location).ok();
        self.setup(&location)?;
        let config = config::load_from_target(flake_path.root(), Path::new(flake_name))
            .context(format!("Failed to load config for {}", options.name))?;
        self.create_bundle(&flake_path, args, &options, &config, &location)?;

        let result =
            if !args.dry_run { self.build(&options, args.target.as_deref(), &location).context("Build failed") } else { Ok(()) };

        let cleanup_mode = mode.cleanup(&flake_path).context("Cleanup failed (temporary flake)");
        if !args.keep {
            println!("Cleaning staging area");
            self.purge(&location).context("Cleanup failed (build files)")?;
            if cleanup_default {
                self.cleanup_default_directory(&location).context("Cleanup failed (build location)")?
            }
        } else {
            println!("Staging area: {}", location.to_string_lossy());
        }

        // Do not short circuit these errors so no temporary data are left lying around
        [result, cleanup_mode].into_iter().collect()
    }

    fn run(&self, mode: Mode) -> Result<()> {
        let options = mode.args().determine_options()?;
        self.execute(options, mode)
    }
}

#[derive(Debug, Subcommand)]
pub enum Subcmd {
    #[clap(flatten)]
    Mode(Box<Mode>),
    /// Compile an existing staging environment
    ///
    /// To generate a staging environment run a command with `--location <somewhere>` and `--keep`,
    /// use `--dry-run` to skip compilation when creating the staging environment
    Compile(Box<Compile>),
    #[clap(hide = true)]
    About,
}

#[derive(Args, Debug)]
pub struct Compile {
    /// Staging environment to compile
    location: PathBuf,
    #[command(flatten)]
    args: BuilderArgs,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Mode {
    /// Package an existing flake
    Flake {
        /// The name of the local flake
        flake_name: String,
        #[command(flatten)]
        args: BuilderArgs,
    },
    /// Package an existing image as a flake
    Image {
        /// The type of pilot/engine to use
        ///
        /// The interpretation of `name` depends on `pilot`
        pilot: String,
        /// The name of the container
        ///
        /// The container may be pulled if it does not exist locally.
        ///
        /// The interpretation of `name` depends on `pilot`
        image_name: String,
        /// The app path of the flake
        // TODO: multi path support
        app: PathBuf,
        #[command(flatten)]
        args: BuilderArgs,
    },
}

impl Mode {
    fn args(&self) -> &BuilderArgs {
        match self {
            Mode::Flake { args, .. } => args,
            Mode::Image { args, .. } => args,
        }
    }

    /// Prepare the flake to be packaged
    ///
    /// Returns the name of the flake
    fn prepare(&self) -> Result<RootedPath> {
        match self {
            Mode::Flake { flake_name, .. } => Ok(flake_name.into()),
            Mode::Image { pilot, image_name, app, args, .. } => {
                // let name = args.options.name.as_deref().unwrap_or(".tmp");
                // let flake_name = format!("{}-{}", name, uuid::Uuid::new_v4().as_hyphenated().to_string());

                let tmp_dir = tempdir()?.into_path();
                std::process::Command::new("flake-ctl")
                    .arg(pilot)
                    .arg("register")
                    .arg("--root")
                    .arg(&tmp_dir)
                    .arg("--app")
                    .arg(app)
                    .arg("--container")
                    .arg(image_name)
                    .args(args.trailing.iter())
                    .status()?;

                Ok(app.with_root(Some(tmp_dir)))
            }
        }
    }

    fn cleanup(&self, path: &RootedPath) -> Result<()> {
        match self {
            Mode::Flake { .. } => {
                // TODO: Once flake modification is implemented the modifications must be cleaned up here
                Ok(())
            }
            Mode::Image { .. } => {
                if let Some(root) = path.root() {
                    remove_dir_all(root)?;
                }
                Ok(())
            }
        }
    }
}

/// Include this with #[command(flatten)] into your args
#[derive(Debug, Args, Clone)]
pub struct BuilderArgs {
    /// Name of the final package
    #[arg(long)]
    pub target: Option<PathBuf>,

    /// Do not actually build the package, just prepare the bundle
    ///
    /// May be used for testing or to create a bundle for a build system such as obs
    #[arg(long)]
    pub dry_run: bool,

    /// Keep the bundle after a dry run
    #[arg(long)]
    pub keep: bool,

    /// Where to build the package;
    ///
    /// defaults to a tmp directory that is deleted afterwards
    #[arg(long)]
    pub location: Option<PathBuf>,

    /// Run in CI (non interactive) mode
    #[arg(long)]
    pub ci: bool,

    #[command(flatten)]
    pub options: PackageOptionsBuilder,

    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    /// Modify the settings of the flake before packaging
    ///
    /// Same as `flake-ctl register <PILOT>` for `image`
    // TODO: enable this if we ever have a "flake modify"
    ///
    /// Ignored for `flake`
    // `flake-ctl modify`
    pub trailing: Vec<String>,

    #[arg(long)]
    /// Do not export the image
    // TODO: This needs a better solution in the future as adherence to this is not enforced right now
    pub skip_export: bool,
}

pub fn export_flake(path: &RootedPath, pilot: &str, bundling_dir: &Path) -> Result<()> {
    let mut cmd = Command::new("flake-ctl");
    cmd.arg(pilot).arg("export");
    if let Some(root) = path.root() {
        cmd.arg("--root").arg(root);
    }

    let out = cmd.arg(path.file_name().unwrap()).arg(bundling_dir).output()?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr))
    }
    Ok(())
}

pub fn copy_configs(path: &RootedPath, bundling_dir: &Path) -> Result<()> {
    let config_path = FLAKE_DIR.join(path.path().file_name().unwrap()).with_root(path.root());
    let configs = [config_path.path_on_disk().with_extension("yaml"), config_path.path_on_disk().with_extension("d")];

    let fake_flake_dir = bundling_dir.join(flakes::config::FLAKE_DIR.strip_prefix("/").unwrap());
    create_dir_all(&fake_flake_dir)?;
    copy_items(&configs, &fake_flake_dir, &CopyOptions::new())?;
    Ok(())
}

#[derive(Parser)]
struct FullArgs<T: FromArgMatches + Args + FlakeBuilder> {
    #[command(subcommand)]
    subcmd: Subcmd,

    #[command(flatten)]
    builder: T,
}

pub fn run<T: FromArgMatches + Args + FlakeBuilder>() -> Result<()> {
    let command = FullArgs::<T>::parse();
    match command.subcmd {
        Subcmd::Mode(mode) => command.builder.run(*mode),
        Subcmd::Compile(comp) => {
            command.builder.build(&comp.args.determine_options()?, comp.args.target.as_deref(), &comp.location)
        }
        Subcmd::About => {
            println!("{};PACKAGER", command.builder.description());
            Ok(())
        }
    }
}
