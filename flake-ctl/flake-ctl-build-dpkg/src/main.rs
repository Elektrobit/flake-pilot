use std::{
    fs::{self, create_dir_all, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Ok, Result};

use flake_ctl_build::{Builder, BuilderInfo, SetupInfo, CompileInfo, options::PackageOptions};
use flakes::{config::{itf::FlakeConfig, FLAKE_DIR}, paths::PathExt};

fn main() -> Result<()> {
    flake_ctl_build::run::<DPKGBuilder>()
}

struct DPKGBuilder;

impl Builder for DPKGBuilder {
    // fn description(&self) -> &str {
    //     "Package flakes with dpkg-deb"
    // }

    fn setup(location: &Path, info: SetupInfo) -> Result<BuilderInfo> {
        infrastructure(location, create_dir_all)?;

        // let edit = !no_edit && !args.ci;
        control_file(&info.options, location, info.config.engine().pilot(), false).context("Failed to create control file")?;
        rules_file(location).context("Failed to create rules file")?;
        install_script(location, &info.config).context("Failed to create install script")?;
        uninstall_script(location, &info.config).context("Failed to create uninstall script")?;
        let export_path = &location.join(flakes::config::FLAKE_DIR.strip_prefix("/").unwrap());
        create_dir_all(export_path)?;
        create_dir_all(location.join("tmp"))?;

        Ok(BuilderInfo {
            image_location: location.join_ignore_abs("tmp"),
            config_location: location.join_ignore_abs(FLAKE_DIR.as_path()),
        })
    }

    fn compile(location: &Path, info: CompileInfo, target: &Path) -> Result<()> {
        Command::new("dpkg-deb")
            .arg("--root-owner-group")
            .arg("--nocheck")
            .arg("-b")
            .arg(location)
            .arg(target.join(info.options.name).with_extension("deb"))
            .status()?;
        Ok(())
    }
}

    fn infrastructure<F, P>(location: P, f: F) -> Result<()>
    where
        F: FnMut(PathBuf) -> Result<(), std::io::Error>,
        P: AsRef<Path>,
    {
        ["tmp", "usr", "DEBIAN"].into_iter().map(|x| location.as_ref().join(x)).try_for_each(f)?;
        Ok(())
    }

    fn control_file(options: &PackageOptions, location: &Path, pilot: &str, edit: bool) -> Result<()> {
        let template = flakes::config::FLAKE_DIR.join("package/dpkg");
        let depends = fs::read_to_string(template.join(pilot))?;

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
        if edit {
            Command::new("vi").arg(location.join("DEBIAN").join("control")).status().context("Failed to open text editor")?;
        }
        Ok(())
    }

    fn rules_file(location: &Path) -> Result<()> {
        OpenOptions::new().create(true).write(true).open(location.join("DEBIAN/rules"))?.write_all("%:\n\tdh $@".as_bytes())?;
        Ok(())
    }

    fn install_script(location: &Path, conf: &FlakeConfig) -> Result<()> {
        let pilot = conf.engine().pilot();
        let mut script = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("postinst"))?;

        let archive = Path::new("/tmp").join(conf.runtime().image_name());
        let archive = archive.to_string_lossy();
        // TODO: Needs to be read from template for other pilots
        script.write_all(format!("podman load < {archive} && rm {archive}\n").as_bytes())?;

        if let Some((first, mut rest)) = conf.runtime().get_symlinks() {
            let first = first.to_string_lossy();
            script.write_all(format!("ln -s /usr/bin/{pilot}-pilot {first}\n").as_bytes())?;
            rest.try_for_each(|path| script.write_all(format!("ln -s {first} {}\n", path.to_string_lossy()).as_bytes()))?;
        }
        Ok(())
    }

    fn uninstall_script(location: &Path, conf: &FlakeConfig) -> Result<()> {
        let mut script = OpenOptions::new().create(true).write(true).open(location.join("DEBIAN").join("prerm"))?;
        script.write_all(format!("podman rmi {}\n", conf.runtime().image_name()).as_bytes())?;
        conf.runtime()
            .paths()
            .keys()
            .map(|x| x.to_string_lossy())
            .try_for_each(|path| script.write_all(format!("rm {path}\n").as_bytes()))?;
        Ok(())
    }
