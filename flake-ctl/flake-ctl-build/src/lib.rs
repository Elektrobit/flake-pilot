use std::{fs::remove_dir_all, io::stdin, env::var};
pub use std::path::{Path, PathBuf};

use anyhow::{Result, anyhow, Context};
use clap::Args;
use flakes::config::{itf::FlakeConfig, self};
use tempfile::tempdir;


pub trait FlakeBuilder {
    /// Create the infrastructure to build the flake package in the given directory
    fn setup(&self, location: &Path) -> Result<()>;

    /// Copy all the necessary files to the build location and create the configuration for the package (e.g. a .spec file)
    fn create_bundle(&self, options: &PackageOptions, config: &FlakeConfig, location: &Path) -> Result<()>;

    /// Perform the actual build process.
    ///
    /// If `target` is given store the resulting package there, otherwise put it into the current working directory
    fn build(&self, options: &PackageOptions, target: Option<&Path>, location: &Path) -> Result<()>;

    /// Remove all the temporary infrastructure created in [setup]
    /// 
    /// This may be run before _and_ after each build
    /// 
    /// Only modify the contents of `location`, if the directory itself needs to be deleted this will be handled by [cleanup_default_directory]
    fn cleanup(&self, location: &Path) -> Result<()>;

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
    fn execute(
        &self, options: &PackageOptions, target: Option<impl AsRef<Path>>, location: Option<impl AsRef<Path>>, build: bool, keep: bool,
    ) -> Result<()> {
        let cleanup_default = location.is_none();
        
        let location = match location.as_ref() {
            Some(location) => location.as_ref().to_owned(),
            None => self.get_default_build_directory()?,
        };
        
        // Clean the location if it is reused, ignore errors here
        self.cleanup(&location).ok();
        self.setup(&location)?;
        let config = config::load_from_name(Path::new(&options.name))?;
        self.create_bundle(options, &config, &location)?;

        if build {
            self.build(options, target.as_ref().map(AsRef::as_ref), &location)?;
        }
        if !keep {
            self.cleanup(&location)?;
            if cleanup_default {
                self.cleanup_default_directory(&location)?
            }
        }
        Ok(())
    }

    fn run(&self, args: &BuilderArgs) -> Result<()> {
        let options = args.determine_options()?;
        self.execute(&options, args.target.as_ref(), args.location.as_ref(), !args.dry_run, args.keep)
    }

    /// Run the packaging except for the actual build step.
    /// Can be used for testing or to create a bundle for a packaging service like obs
    ///
    /// [cleanup] is not run if `keep` is `true`
    fn dry_run(&self, options: &PackageOptions, location: Option<impl AsRef<Path>>, keep: bool) -> Result<()>{
        self.execute(options, Option::<PathBuf>::None, location, false, keep)
    }
}

#[derive(Debug)]
pub struct PackageOptions{
    pub name: String,
    pub description: String,
    pub version: String,
    pub url: String,
    pub maintainer: String,
    pub license: String
}

#[derive(Debug, Default, Args, Clone)]
pub struct PackageOptionsBuilder{
    #[arg(long)]
    pub name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    pub url: Option<String>,
    #[arg(long)]
    pub maintainer: Option<String>,
    #[arg(long)]
    pub license: Option<String>
}

impl PackageOptionsBuilder {
    pub fn build(self) -> Result<PackageOptions> {
        Ok(PackageOptions {
            name: self.name.ok_or_else(||anyhow!("Missing package name"))?,
            description: self.description.ok_or_else(||anyhow!("Missing package description"))?,
            version: self.version.ok_or_else(||anyhow!("Missing package version"))?,
            url: self.url.ok_or_else(||anyhow!("Missing package url"))?,
            maintainer: self.maintainer.ok_or_else(||anyhow!("Missing package maintainer"))?,
            license: self.license.ok_or_else(||anyhow!("Missing package license"))?,
        })
    }
}

/// Include this with #[command(flatten)] into your args
#[derive(Args)]
pub struct BuilderArgs {
    /// Package a local flake
    #[arg(long)]
    pub flake: Option<String>,
    /// Package a remote image
    ///
    /// The formatting of this parameter depends on the type of pilot used
    #[arg(long)]
    pub source: Option<String>,
    /// The type of pilot used in the package
    ///
    /// Must match with --source if specified.
    ///
    /// Can be omitted for --flake but must match if given
    #[arg(long)]
    pub pilot: Option<String>,

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

    /// Where to build the package; default: tmp directory
    #[arg(long)]
    pub location: Option<String>,

    /// Run in CI (non interactive) mode
    #[arg(long)]
    pub ci: bool,

    #[command(flatten)]
    pub options: PackageOptionsBuilder,

    #[arg(trailing_var_arg = true)]
    /// Modify the settings of the flake before packaging
    ///
    /// Same as `flake-ctl register <PILOT>` if --source is given
    // TODO: enable this if we ever have a "flake modify"
    ///
    /// Ignored if --flake is given
    // `flake-ctl modify` if --flake is given
    pub trailing: Vec<String>,
}

impl BuilderArgs {

    pub fn determine_options(&self) -> Result<PackageOptions> {
        let mut options = self.options.clone();
        
        // Read from env where not given
        options.name = options.name.or_else(|| var("PKG_FLAKE_NAME").ok());
        options.description = options.description.or_else(|| var("PKG_FLAKE_DESCRIPTION").ok());
        options.version = options.version.or_else(|| var("PKG_FLAKE_VERSION").ok());
        options.url = options.url.or_else(|| var("PKG_FLAKE_URL").ok());
        options.maintainer = options.maintainer.or_else(|| var("PKG_FLAKE_MAINTAINER").ok());
        options.license = options.license.or_else(|| var("PKG_FLAKE_LICENSE").ok());
    
        if !self.ci {
            options.name = options.name.or_else(|| user_input("Name").ok());
            options.description = options.description.or_else(|| user_input("Description").ok());
            options.version = options.version.or_else(|| user_input("Version").ok());
            options.url = options.url.or_else(|| user_input("URL").ok());
            options.maintainer = options.maintainer.or_else(|| user_input("Maintainer").ok());
            options.license = options.license.or_else(|| user_input("License").ok());
        }
    
        options.build().context("Missing packaging option")
    }
    
    }
    
    fn user_input(name: &str) -> Result<String> {
        let mut buf = String::new();
        println!("{name}: ");
        stdin().read_line(&mut buf)?;
        Ok(buf.trim_end().to_owned())
    }