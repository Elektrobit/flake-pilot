use anyhow::{Context, Result};
use clap::{builder::OsStr, Args, ValueEnum};
use flakes::{config::{self, itf::FlakeConfig}, paths::RootedPath};
use fs_extra::{copy_items, dir::CopyOptions};
use tempfile::tempdir_in;

use flake_ctl_build::{export_flake, FlakeBuilder, PackageOptions, copy_configs};

fn main() -> Result<()> {
    flake_ctl_build::run::<RPMBuilder>()
}

use std::{
    fs::{self, copy, create_dir_all, remove_dir_all, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Args)]
pub struct RPMBuilder {
    /// Location of .spec template and pilot specific data
    #[arg(long, default_value = "/usr/share/flakes/package/rpmbuild")]
    template: PathBuf,

    /// skip spec editing
    #[arg(long)]
    no_edit: bool,

    /// What toolchain to use for building
    #[arg(long, value_enum, default_value = Tooling::RPMBuild)]
    tooling: Tooling,
}

#[derive(Debug, Clone, ValueEnum, Copy)]
#[value(rename_all = "lower")]
pub enum Tooling {
    RPMBuild,
    DebBuild,
}

impl From<Tooling> for OsStr {
    fn from(value: Tooling) -> Self {
        match value {
            Tooling::RPMBuild => "rpmbuild".into(),
            Tooling::DebBuild => "debbuild".into(),
        }
    }
}

impl FlakeBuilder for RPMBuilder {
    fn setup(&self, location: &Path) -> Result<()> {
        self.infrastructure(location, create_dir_all)
    }

    fn create_bundle(&self, flake_path: &RootedPath, options: &PackageOptions, config: &FlakeConfig, location: &Path) -> Result<()> {
        let PackageOptions { name, version, .. } = options;
        let temp_dir = tempdir_in(location).context("Failed to create bundling dir")?;
        let bundling_dir = temp_dir.path().join(format!("{name}-{version}"));

        create_dir_all(&bundling_dir)?;

        self.copy_includes(config, &bundling_dir).context("Failed to copy includes to bundling dir")?;
        copy_configs(flake_path, &bundling_dir).context("Failed to copy configs to bundling dir")?;
        export_flake(flake_path, config.engine().pilot(), &bundling_dir).context("Failed to export flake image(s)")?;
        self.compress_bundle(temp_dir.path(), name, version).context("Failed to compress bundle")?;
        copy(bundling_dir.with_extension("tar.gz"), location.join("SOURCES").join(name).with_extension("tar.gz"))
            .context("Failed to move bundle to build dir")?;

        self.create_spec(flake_path, &location.join("SPECS"), options, config).context("Failed to create spec")?;

        Ok(())
    }

    fn build(&self, options: &PackageOptions, target: Option<&Path>, location: &Path) -> Result<()> {
        Command::new(OsStr::from(self.tooling))
            .arg("-ba")
            .arg("--define")
            .arg(format!("_topdir {}", location.to_string_lossy()))
            .arg(location.join("SPECS").join(&options.name).with_extension("spec"))
            .status()?;

        let package_dir = match self.tooling {
            Tooling::RPMBuild => "RPMS",
            Tooling::DebBuild => "DEBS",
        };
        copy_items(&[location.join(package_dir)], target.unwrap_or_else(|| Path::new(".")), &CopyOptions::default())?;
        Ok(())
    }

    fn cleanup(&self, location: &Path) -> Result<()> {
        self.infrastructure(location, remove_dir_all)
    }
}

impl RPMBuilder {
    fn infrastructure<F, P>(&self, location: P, f: F) -> Result<()>
    where
        F: FnMut(PathBuf) -> Result<(), std::io::Error>,
        P: AsRef<Path>,
    {
        ["BUILD", "SOURCES", "SPECS"]
            .into_iter()
            .chain(match self.tooling {
                Tooling::RPMBuild => ["RPMS", "SRPMS"],
                Tooling::DebBuild => ["DEBS", "SDEBS"],
            })
            .map(|x| location.as_ref().join(x))
            .try_for_each(f)?;
        Ok(())
    }

    fn copy_includes(&self, config: &FlakeConfig, bundling_dir: &Path) -> Result<()> {
        let bundles = config.static_data().get_bundles().into_iter().flatten();
        for tar in bundles {
            copy(tar, bundling_dir.join(tar))?;
        }
        Ok(())
    }

    fn compress_bundle(&self, bundling_dir: &Path, name: &str, version: &str) -> Result<()> {
        Command::new("tar")
            .arg("-czvf")
            .arg(bundling_dir.join(format!("{name}-{version}")).with_extension("tar.gz"))
            .arg("-C")
            .arg(bundling_dir)
            .arg(".")
            .status()?;
        Ok(())
    }

    fn create_spec(&self, flake_path: &RootedPath, path: &Path, options: &PackageOptions, config: &FlakeConfig) -> Result<()> {
        let spec_path = path.join(&options.name).with_extension("spec");
        let mut spec = OpenOptions::new().write(true).create_new(true).open(&spec_path)?;
        let content = self.construct_spec(&flake_path.file_name().unwrap().to_string_lossy(), config, options)?;
        spec.write_all(content.as_bytes())?;
        spec.flush()?;

        // TODO: Select default text editor
        if !self.no_edit {
            Command::new("vi").arg(&spec_path).status().context("Failed to open text editor")?;
        }
        Ok(())
    }

    fn construct_spec(&self, flake_name: &str, config: &FlakeConfig, options: &PackageOptions) -> Result<String> {
        let mut template =
            fs::read_to_string(self.template.join("template").with_extension("spec")).context("Failed to read spec template")?;

        // TODO: maybe use faster templating here
        // TODO: Drop line with empty information completely
        let data = self.template.join(config.engine().pilot());
        let requires = fs::read_to_string(&data).context(format!("Failed to load pilot specific data, {data:?}"))?;

        let vals = [
            ("%{_flake_name}", flake_name),
            ("%{_flake_package_name}", options.name.as_str()),
            ("%{_flake_version}", options.version.as_str()),
            ("%{_flake_desc}", options.description.as_str()),
            ("%{_flake_url}", options.url.as_str()),
            ("%{_flake_license}", options.license.as_str()),
            ("%{_flake_maintainer_name}", options.maintainer_name.as_str()),
            ("%{_flake_maintainer_email}", options.maintainer_email.as_str()),
            ("%{_flake_pilot}", config.engine().pilot()),
            ("%{_flake_requires}", &requires),
            ("%{_flake_dir}", &config::FLAKE_DIR.to_string_lossy()),
        ];

        for (placeholder, value) in vals {
            template = template.replace(placeholder, value);
        }

        // TODO: multiple links

        Ok(template)
    }
}
