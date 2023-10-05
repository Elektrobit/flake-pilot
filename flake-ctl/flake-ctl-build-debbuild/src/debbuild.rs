use std::{
    fs::{self, copy, create_dir_all, OpenOptions, remove_dir_all},
    io::Write,
    path::Path,
    process::Command,
};

use anyhow::{Context, Result};
use flakes::config::itf::FlakeConfig;
use fs_extra::{copy_items, dir::CopyOptions};
use tempfile::tempdir_in;

use flake_ctl_build::{FlakeBuilder, PackageOptions};

pub struct Builder<'a> {
    pub template: &'a Path,
    pub edit: bool,
}

impl<'a> FlakeBuilder for Builder<'a> {
    fn setup(&self, location: &Path) -> Result<()> {
        create_dir_all(location.join("BUILD"))?;
        create_dir_all(location.join("SOURCES"))?;
        create_dir_all(location.join("DEBS"))?;
        create_dir_all(location.join("SDEBS"))?;
        create_dir_all(location.join("SPECS"))?;
        Ok(())
    }

    fn create_bundle(&self, options: &PackageOptions, config: &FlakeConfig, location: &Path) -> Result<()> {
        let PackageOptions { name, version, .. } = options;
        let temp_dir = tempdir_in(location).context("Failed to create bundling dir")?;
        let bundling_dir = temp_dir.path().join(format!("{name}-{version}"));

        println!("{:?}", bundling_dir);

        create_dir_all(&bundling_dir)?;

        self.copy_includes(config, &bundling_dir).context("Failed to copy includes to bundling dir")?;
        self.copy_configs(name, &bundling_dir).context("Failed to copy configs to bundling dir")?;
        self.export_flake(name, config.engine().pilot(), &bundling_dir).context("Failed to export flake image(s)")?;
        self.compress_bundle(temp_dir.path(), name, version).context("Failed to compress bundle")?;
        copy(bundling_dir.with_extension("tar.gz"), location.join("SOURCES").join(name).with_extension("tar.gz"))
            .context("Failed to move bundle to build dir")?;

        self.create_spec(&location.join("SPECS"), options, config).context("Failed to create spec")?;

        Ok(())
    }

    fn build(&self, options: &PackageOptions, target: Option<&Path>, location: &Path) -> Result<()> {
        Command::new("debbuild")
            .arg("-ba")
            .arg("--define")
            .arg(format!("_topdir {}", location.to_string_lossy()))
            .arg(location.join("SPECS").join(&options.name).with_extension("spec"))
            .status()?;
        if let Some(target) = target {
            copy_items(&[location.join("DEBS/all")], target, &CopyOptions::default())?;
        } else {
            copy_items(&[location.join("DEBS/all")], ".", &CopyOptions::default())?;
        }
        Ok(())
    }

    fn cleanup(&self, location: &Path) -> Result<()> {
        remove_dir_all(location.join("BUILD"))?;
        remove_dir_all(location.join("SOURCES"))?;
        remove_dir_all(location.join("DEBS"))?;
        remove_dir_all(location.join("SDEBS"))?;
        remove_dir_all(location.join("SPECS"))?;
        Ok(())
    }
}

impl<'a> Builder<'a> {
    fn copy_includes(&self, config: &FlakeConfig, bundling_dir: &Path) -> Result<()> {
        let bundles = config.static_data().get_bundles().into_iter().flatten();
        for tar in bundles {
            copy(tar, bundling_dir.join(tar))?;
        }
        Ok(())
    }
    fn copy_configs(&self, name: &str, bundling_dir: &Path) -> Result<()> {
        let config_path = Path::new("/usr/share/flakes").join(name);
        let configs = [config_path.with_extension("yaml"), config_path.with_extension("d")];

        let fake_flake_dir = bundling_dir.join("usr/share/flakes");
        create_dir_all(&fake_flake_dir)?;
        copy_items(&configs, &fake_flake_dir, &CopyOptions::new())?;
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

    fn create_spec(&self, path: &Path, options: &PackageOptions, config: &FlakeConfig) -> Result<()> {
        let spec_path = path.join(&options.name).with_extension("spec");
        let mut spec = OpenOptions::new().write(true).create_new(true).open(&spec_path)?;
        let content = self.construct_spec(config, options)?;
        spec.write_all(content.as_bytes())?;
        spec.flush()?;

        // TODO: Select default text editor
        if self.edit {
            Command::new("vi").arg(&spec_path).status().context("Failed to open text editor")?;
        }
        Ok(())
    }

    fn construct_spec(&self, config: &FlakeConfig, options: &PackageOptions) -> Result<String> {
        let template = fs::read_to_string(self.template)?;

        // TODO: maybe use faster templating here
        let template = template.replace("%{_flake_name}", &options.name);
        let template = template.replace("%{_flake_version}", &options.version);
        let template = template.replace("%{_flake_desc}", &options.description);
        let template = template.replace("%{_flake_url}", &options.url);
        let template = template.replace("%{_flake_license}", &options.license);
        let template = template.replace("%{_flake_maintainer}", &options.maintainer);
        let template = template.replace("%{_flake_pilot}", config.engine().pilot());

        let data = Path::new("/usr/share/flakes/package/debbuild/").join(config.engine().pilot());

        let requires = fs::read_to_string(&data).context(format!("Failed to load pilot specific data, {:?}", data))?;
        let template = template.replace("%{_flake_requires}", &requires);

        // TODO: multiple links

        Ok(template)
    }

    fn export_flake(&self, name: &str, pilot: &str, bundling_dir: &Path) -> Result<()> {
        Command::new("flake-ctl").arg(pilot).arg("export").arg(name).arg(bundling_dir.join(name)).status()?;
        Ok(())
    }
}
