use std::{
    fs::{self, create_dir_all, remove_dir_all, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Ok, Result};
use clap::Args;
use flake_ctl_build::{copy_configs, export_flake, FlakeBuilder, PackageOptions};
use flakes::{config::itf::FlakeConfig, paths::RootedPath};

fn main() -> Result<()> {
    flake_ctl_build::run::<DPKGBuilder>()
}

#[derive(Debug, Args)]
struct DPKGBuilder {
    /// Location of pilot specific data
    #[arg(long, default_value = flakes::config::FLAKE_DIR.join("package/dpkg").into_os_string())]
    template: PathBuf,

    /// skip control editing
    #[arg(long)]
    no_edit: bool,
}

impl FlakeBuilder for DPKGBuilder {
    fn description(&self) -> &str {
        "Package flakes with dpkg-deb"
    }

    fn setup(&self, location: &Path) -> Result<()> {
        self.infrastructure(location, create_dir_all)
    }

    fn create_bundle(
        &self, flake_name: &RootedPath, options: &PackageOptions, config: &FlakeConfig, location: &Path,
    ) -> Result<()> {
        self.control_file(options, location, config.engine().pilot()).context("Failed to create control file")?;
        self.rules_file(location).context("Failed to create rules file")?;
        self.install_script(flake_name, location, config).context("Failed to create install script")?;
        self.uninstall_script(location, config).context("Failed to create uninstall script")?;
        let export_path = &location.join(flakes::config::FLAKE_DIR.strip_prefix("/").unwrap());
        create_dir_all(export_path)?;
        create_dir_all(location.join("tmp"))?;
        export_flake(flake_name, config.engine().pilot(), &location.join("tmp")).context("Failed to export flake")?;
        copy_configs(flake_name, location)?;
        Ok(())
    }

    fn build(&self, options: &PackageOptions, target: Option<&Path>, location: &Path) -> Result<()> {
        Command::new("dpkg-deb")
            .arg("--root-owner-group")
            .arg("--nocheck")
            .arg("-b")
            .arg(location)
            .arg(target.unwrap_or_else(|| Path::new(".")).join(&options.name).with_extension("deb"))
            .status()?;
        Ok(())
    }

    fn purge(&self, location: &Path) -> anyhow::Result<()> {
        self.infrastructure(location, remove_dir_all)
    }
}

impl DPKGBuilder {
    fn infrastructure<F, P>(&self, location: P, f: F) -> Result<()>
    where
        F: FnMut(PathBuf) -> Result<(), std::io::Error>,
        P: AsRef<Path>,
    {
        ["bin", "tmp", "etc", "usr", "DEBIAN"].into_iter().map(|x| location.as_ref().join(x)).try_for_each(f)?;
        Ok(())
    }

    fn control_file(&self, options: &PackageOptions, location: &Path, pilot: &str) -> Result<()> {
        let depends = fs::read_to_string(self.template.join(pilot))?;

        let mut cfile = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("control"))?;

        [
            ("Section", "other"),
            ("Priority", "optional"),
            ("Maintainer", &format!("\"{}\" <{}>", options.maintainer_name, options.maintainer_email)),
            ("Homepage", &options.url),
            ("Package", &options.name),
            ("Architecture", "all"),
            ("Multi-Arch", "foreign"),
            ("Depends", &depends),
            ("Description", &options.description),
            ("Version", &options.version),
            ("Package-Type", "deb"),
            ("Rules-Requires-Root", "binary-targets"),
        ]
        .into_iter()
        .try_for_each(|(name, value)| cfile.write_all(format!("{name}: {value}\n").as_bytes()))?;

        // TODO: Select default text editor
        if !self.no_edit {
            Command::new("vi").arg(location.join("DEBIAN").join("control")).status().context("Failed to open text editor")?;
        }
        Ok(())
    }

    fn rules_file(&self, location: &Path) -> Result<()> {
        OpenOptions::new().create(true).write(true).open(location.join("DEBIAN/rules"))?.write_all("%:\n\tdh $@".as_bytes())?;
        Ok(())
    }

    fn install_script(&self, flake_name: &RootedPath, location: &Path, conf: &FlakeConfig) -> Result<()> {
        let pilot = conf.engine().pilot();
        let mut script = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("postinst"))?;
        
        let name = flake_name.file_name().unwrap_or_default().to_string_lossy();
        // TODO: Needs to be read from template for other pilots
        script.write_all(format!("podman load < /tmp/{name}\n").as_bytes())?;

        if let Some((first, mut rest)) = conf.runtime().get_symlinks() {
            let first = first.to_string_lossy();
            script.write_all(format!("ln -s /usr/bin/{pilot}-pilot {first}\n").as_bytes())?;
            rest.try_for_each(|path| script.write_all(format!("ln -s {first} {}\n", path.to_string_lossy()).as_bytes()))?;
        }
        Ok(())
    }

    fn uninstall_script(&self, location: &Path, conf: &FlakeConfig) -> Result<()> {
        let mut script = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("prerm"))?;
        script.write_all(format!("podman rmi {}\n", conf.runtime().image_name()).as_bytes())?;
        conf.runtime()
            .paths()
            .keys()
            .map(|x| x.to_string_lossy())
            .try_for_each(|path| script.write_all(format!("rm {path}\n").as_bytes()))?;
        Ok(())
    }
}
