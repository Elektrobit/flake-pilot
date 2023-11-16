use std::{
    ffi::OsStr,
    fs::{copy, create_dir_all, read_dir},
    path::Path,
    process::{Command, Stdio},
};

use anyhow::{bail, Context, Ok, Result};
use colored::Colorize;
use flakes::config::load_from_path;
use fs_extra::{copy_items, dir::CopyOptions};
use termion::clear;

pub(crate) fn build() -> Result<()> {
    let config = load_from_path(Path::new("src/flake"))?;
    let name = config.runtime().get_symlinks().unwrap().0.file_name().unwrap();

    print!("{}", " Setup... ".yellow().bold());
    setup(name).context("Setup failed")?;
    println!("{}\r{}", clear::CurrentLine, "Setup".green().bold());

    print!("{}", " Building Image... ".yellow().bold());
    let method = discover_image_builder()?;
    let name = "image_name";
    method.build(name)?;
    println!("{}\r{} ({})", clear::CurrentLine, "Built Image".green().bold(), name);

    print!("{}", " Compiling... ".yellow().bold());
    Command::new("flake-ctl").arg("build").arg("compile").arg(".staging").arg("--target").arg("out").output()?;
    println!("{}\r{}", clear::CurrentLine, "Compiled".green().bold());

    println!("{} ({})", "Build finished".green().bold(), name);
    Ok(())
}

fn setup(name: &OsStr) -> Result<()> {
    create_dir_all("out").context("could not create output directory")?;

    let staging = Path::new(".staging/usr/share/flakes");
    copy("src/flake.yaml", staging.join(name).with_extension("yaml")).context("No flake.yaml")?;
    copy("src/options.yaml", ".flakes/package/options.yaml").context("No flake.yaml")?;
    copy_items(&["src/flake.d"], staging.join(name).with_extension("d"), &CopyOptions::default()).ok();
    Ok(())
}

enum ImageBuilder {
    Dockerfile,
    Script,
}

fn discover_image_builder() -> Result<ImageBuilder> {
    let mut i = read_dir("src")?.filter_map(Result::ok).filter_map(|f| match f.file_name().to_str() {
        Some("Dockerfile") => Some(ImageBuilder::Dockerfile),
        Some("build.sh") => Some(ImageBuilder::Script),
        _ => None,
    });
    let found = i.next();
    if i.next().is_some() {
        bail!("Discovered more than one image build method")
    }
    if let Some(found) = found {
        Ok(found)
    } else {
        bail!("No image build method discovered")
    }
}

impl ImageBuilder {
    fn build(&self, name: &str) -> Result<String> {
        match self {
            ImageBuilder::Dockerfile => todo!(),
            ImageBuilder::Script => {
                let out = Command::new("src/build.sh").arg(name).stderr(Stdio::inherit()).output().context("Failed to run src/build.sh")?;
                if !out.status.success() {
                    bail!("Failed to build image with build.sh")
                }
            }
        }
        Ok(name.to_owned())
    }
}
