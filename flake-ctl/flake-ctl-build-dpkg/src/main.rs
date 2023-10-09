use std::{
    fs::{self, create_dir_all, remove_dir_all, OpenOptions},
    io::Write,
    process::Command,
};

use anyhow::{Context, Ok, Result};
use clap::Args;
use flake_ctl_build::{copy_configs, export_flake, FlakeBuilder, PackageOptions, Path, PathBuf};
use flakes::config::itf::FlakeConfig;

fn main() -> Result<()> {
    flake_ctl_build::run::<DPKGBuilder>()
}

#[derive(Debug, Args)]
struct DPKGBuilder {
    /// Location of .spec template and pilot specific data
    #[arg(long, default_value = "/usr/share/flakes/package/dpkg")]
    template: PathBuf,

    /// skip spec editing
    #[arg(long)]
    no_edit: bool,
}

impl FlakeBuilder for DPKGBuilder {
    fn setup(&self, location: &flake_ctl_build::Path) -> anyhow::Result<()> {
        self.infrastructure(location, create_dir_all)
    }

    fn create_bundle(
        &self, options: &flake_ctl_build::PackageOptions, config: &flakes::config::itf::FlakeConfig,
        location: &flake_ctl_build::Path,
    ) -> anyhow::Result<()> {
        self.control_file(options, location, config.engine().pilot()).context("Failed to create control file")?;
        self.rules_file(location).context("Failed to create rules file")?;
        self.install_script(location, config).context("Failed to create install script")?;
        self.uninstall_script(location, config).context("Failed to create uninstall script")?;
        let export_path = &location.join("usr/share/flakes");
        create_dir_all(export_path)?;
        export_flake(&options.name, config.engine().pilot(), export_path).context("Failed to export flake")?;
        copy_configs(&options.name, location)?;
        Ok(())
    }

    fn build(
        &self, options: &flake_ctl_build::PackageOptions, target: Option<&flake_ctl_build::Path>,
        location: &flake_ctl_build::Path,
    ) -> anyhow::Result<()> {
        Command::new("dpkg-deb")
            .arg("--root-owner-group")
            .arg("--nocheck")
            .arg("-b")
            .arg(location)
            .arg(target.unwrap_or_else(|| Path::new(".")).join(&options.name).with_extension("deb"))
            .status()?;
        Ok(())
    }

    fn cleanup(&self, location: &flake_ctl_build::Path) -> anyhow::Result<()> {
        self.infrastructure(location, remove_dir_all)
    }
}

impl DPKGBuilder {
    fn infrastructure<F, P>(&self, location: P, f: F) -> Result<()>
    where
        F: FnMut(PathBuf) -> Result<(), std::io::Error>,
        P: AsRef<Path>,
    {
        ["bin", "etc", "usr", "DEBIAN"].into_iter().map(|x| location.as_ref().join(x)).try_for_each(f)?;
        Ok(())
    }

    fn control_file(&self, options: &PackageOptions, location: &Path, pilot: &str) -> Result<()> {
        let depends = fs::read_to_string(self.template.join(pilot))?;

        let mut cfile = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("control"))?;

        [
            ("Section", "other"),
            ("Priority", "optional"),
            ("Maintainer", &format!("\"{}\", <{}>", options.maintainer_name, options.maintainer_email)),
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
        Ok(())
    }

    fn rules_file(&self, location: &Path) -> Result<()> {
        OpenOptions::new().create(true).write(true).open(location.join("DEBIAN/rules"))?.write_all("%:\n\tdh $@".as_bytes())?;
        Ok(())
    }

    fn install_script(&self, location: &Path, conf: &FlakeConfig) -> Result<()> {
        let pilot = conf.engine().pilot();
        let mut script = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("postinst"))?;
        conf.runtime()
            .paths()
            .values()
            .map(|x| x.exports().to_string_lossy())
            .try_for_each(|path| script.write_all(format!("ln /usr/bin/{pilot}-pilot {path}\n").as_bytes()))?;
        Ok(())
    }

    fn uninstall_script(&self, location: &Path, conf: &FlakeConfig) -> Result<()> {
        let mut script = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("prerm"))?;
        conf.runtime()
            .paths()
            .values()
            .map(|x| x.exports().to_string_lossy())
            .try_for_each(|path| script.write_all(format!("rm {path}\n").as_bytes()))?;
        Ok(())
    }
}
