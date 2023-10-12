use std::{env::var, io::stdin};

use anyhow::{Result, anyhow, Context};
use clap::Args;

use crate::BuilderArgs;


fn user_input(name: &str) -> Result<String> {
    let mut buf = String::new();
    println!("{name}: ");
    stdin().read_line(&mut buf)?;
    Ok(buf.trim_end().to_owned())
}

impl BuilderArgs {
    pub fn determine_options(&self) -> Result<PackageOptions> {
        let mut options = self.options.clone();

        // Read from env where not given
        options.name = options.name.or_else(|| var("PKG_FLAKE_NAME").ok());
        options.description = options.description.or_else(|| var("PKG_FLAKE_DESCRIPTION").ok());
        options.version = options.version.or_else(|| var("PKG_FLAKE_VERSION").ok());
        options.url = options.url.or_else(|| var("PKG_FLAKE_URL").ok());
        options.maintainer_name = options.maintainer_name.or_else(|| var("PKG_FLAKE_MAINTAINER_NAME").ok());
        options.maintainer_email = options.maintainer_email.or_else(|| var("PKG_FLAKE_MAINTAINER_EMAIL").ok());
        options.license = options.license.or_else(|| var("PKG_FLAKE_LICENSE").ok());

        if !self.ci {
            options.name = options.name.or_else(|| user_input("Name").ok());
            options.description = options.description.or_else(|| user_input("Description").ok());
            options.version = options.version.or_else(|| user_input("Version").ok());
            options.url = options.url.or_else(|| user_input("URL").ok());
            options.maintainer_name = options.maintainer_name.or_else(|| user_input("Maintainer Name").ok());
            options.maintainer_email = options.maintainer_email.or_else(|| user_input("Maintainer Email").ok());
            options.license = options.license.or_else(|| user_input("License").ok());
        }

        options.build().context("Missing packaging option")
    }
}

#[derive(Debug)]
pub struct PackageOptions {
    pub name: String,
    pub description: String,
    pub version: String,
    pub url: String,
    pub maintainer_name: String,
    pub maintainer_email: String,
    pub license: String,
}

#[derive(Debug, Default, Args, Clone)]
pub struct PackageOptionsBuilder {
    #[arg(long)]
    /// The name of the package (excluding version, arch, etc.)
    pub name: Option<String>,
    #[arg(long)]
    pub description: Option<String>,
    #[arg(long)]
    pub version: Option<String>,
    #[arg(long)]
    /// A url pointing to the packages source
    pub url: Option<String>,
    #[arg(long)]
    pub maintainer_name: Option<String>,
    #[arg(long)]
    pub maintainer_email: Option<String>,
    #[arg(long)]
    pub license: Option<String>,
}

impl PackageOptionsBuilder {
    pub fn build(self) -> Result<PackageOptions> {
        Ok(PackageOptions {
            name: self.name.ok_or_else(|| anyhow!("Missing package name"))?,
            description: self.description.ok_or_else(|| anyhow!("Missing package description"))?,
            version: self.version.ok_or_else(|| anyhow!("Missing package version"))?,
            url: self.url.ok_or_else(|| anyhow!("Missing package url"))?,
            maintainer_name: self.maintainer_name.ok_or_else(|| anyhow!("Missing package maintainer name"))?,
            maintainer_email: self.maintainer_email.ok_or_else(|| anyhow!("Missing package maintainer email"))?,
            license: self.license.ok_or_else(|| anyhow!("Missing package license"))?,
        })
    }
}
