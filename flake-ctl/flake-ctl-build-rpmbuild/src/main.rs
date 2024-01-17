

use anyhow::{Context, Result, Ok};
use clap::{builder::OsStr, Args, ValueEnum};
use flake_ctl_build::{about, Builder, options::PackageOptions, BuilderInfo, SetupInfo};
use flakes::{config::{self, itf::FlakeConfig}, paths::PathExt};
use fs_extra::{copy_items, dir::CopyOptions};
use tempfile::tempdir_in;

// use flake_ctl_build::{export_flake, FlakeBuilder, PackageOptions, copy_configs, BuilderArgs};
use flakes::config::FLAKE_DIR;

fn main() -> Result<()> {
    about("Package flakes with rpmbuild");
    flake_ctl_build::run::<RPMBuilder>()
}

use std::{
    fs::{self, copy, create_dir_all, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Args)]
pub struct RPMBuilder {
    /// Location of .spec template and pilot specific data
    #[arg(long, default_value = FLAKE_DIR.join("package/rpmbuild").into_os_string())]
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

impl Builder for RPMBuilder {

    fn setup(&self, location: &std::path::Path, info: flake_ctl_build::SetupInfo) -> Result<flake_ctl_build::BuilderInfo> {
        self.infrastructure(location, create_dir_all)?;

        self.create_spec(location, &info, false).context("Failed to create spec")?;

        let image_location = location.join_ignore_abs("tmp");
        create_dir_all(&image_location)?;

        let config_location = location.join_ignore_abs(FLAKE_DIR.as_path());
        create_dir_all(&config_location)?;

        Ok(BuilderInfo {
            image_location,
            config_location
        })
    }

    fn compile(&self, location: &std::path::Path, info: flake_ctl_build::CompileInfo, target: &std::path::Path) -> Result<()> {

        let PackageOptions { name, version, .. } = &info.options;
        let temp_dir = tempdir_in(location).context("Failed to create bundling dir")?;
        let bundling_dir = temp_dir.path().join(format!("{name}-{version}"));

        create_dir_all(&bundling_dir)?;

        self.copy_includes(&info.config, &bundling_dir).context("Failed to copy includes to bundling dir")?;
        self.compress_bundle(temp_dir.path(), name, version).context("Failed to compress bundle")?;
        copy(bundling_dir.with_extension("tar.gz"), location.join("SOURCES").join(name).with_extension("tar.gz"))
            .context("Failed to move bundle to build dir")?;

        Command::new(OsStr::from(self.tooling))
            .arg("-ba")
            .arg("-vv")
            .arg("--define")
            .arg(format!("_topdir {}", location.canonicalize()?.to_string_lossy()))
            .arg(location.join("SPECS").join(info.options.name).with_extension("spec"))
            .status().context(format!("Failed to run {:?}", self.tooling))?;

        let package_dir = match self.tooling {
            Tooling::RPMBuild => "RPMS",
            Tooling::DebBuild => "DEBS",
        };
        copy_items(&[location.join(package_dir)], target, &CopyOptions::default()).context("Failed to copy build result")?;
        Ok(())
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

    fn create_spec(&self, location: &Path, info: &SetupInfo, edit: bool) -> Result<()> {
    // fn create_spec(&self, flake_path: &Path, path: &Path, options: &PackageOptions, config: &FlakeConfig, edit: bool) -> Result<()> {
        let spec_path = location.join("SPECS").join(&info.options.name).with_extension("spec");
        let mut spec = OpenOptions::new().write(true).create_new(true).open(&spec_path)?;
        let content = self.construct_spec(&info.flake_name, &info.config, &info.options)?;
        spec.write_all(content.as_bytes())?;
        spec.flush()?;

        // TODO: Select default text editor
        if edit {
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
        
        let mut symlinks: Vec<String> = vec![];
        if let Some((first, rest)) = config.runtime().get_symlinks() {
            let first = first.to_string_lossy();
            symlinks.push(format!("ln -s %{{_bindir}}/%{{_flake_pilot}}-pilot {first}"));
            symlinks.extend(rest.map(|path| format!("ln {first} {}", path.to_string_lossy())))
        }
        let link_create = symlinks.join("\n");

        let link_remove = config
            .runtime()
            .paths()
            .keys()
            .map(|p| format!("rm {}", p.to_string_lossy()))
            .collect::<Vec<_>>()
            .join("\n");

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
            ("%{_flake_links_create}", &link_create),
            ("%{_flake_links_remove}", &link_remove),
            ("%{_flake_image_tag}", config.runtime().image_name()),
        ];

        for (placeholder, value) in vals {
            template = template.replace(placeholder, value);
        }

        Ok(template)
    }
}
