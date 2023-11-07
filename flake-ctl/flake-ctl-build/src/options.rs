use std::{env::var, io::stdin};

use anyhow::{anyhow, Context, Result};
use clap::Args;
use serde::Deserialize;

use crate::{BuilderArgs, config::{get_global, get_local}};


fn user_input(name: &str) -> Result<String> {
    let mut buf = String::new();
    println!("{name}: ");
    stdin().read_line(&mut buf)?;
    Ok(buf.trim_end().to_owned())
}

macro_rules! fill_in {
    ($options:ident : $($name:ident = $getter:expr;)*) => {
        $($options.$name = $options.$name.or_else(|| $getter.ok());)*
    };
}

impl BuilderArgs {
    pub fn determine_options(&self) -> Result<PackageOptions> {
        let mut options = PackageOptionsBuilder::default();

        // Get options from global/local settings
        if let Ok(global) = get_global() {
            options = options.update(global);
        }
        if let Ok(local) = get_local() {
            options = options.update(local);
        }

        // Options on CLI override global/local settings
        options = options.update(self.options.clone());

        // Read from env where not given
        fill_in!(options:
            name = var("PKG_FLAKE_NAME");
            description = var("PKG_FLAKE_DESCRIPTION");
            version = var("PKG_FLAKE_VERSION");
            url = var("PKG_FLAKE_URL");
            maintainer_name = var("PKG_FLAKE_MAINTAINER_NAME");
            maintainer_email = var("PKG_FLAKE_MAINTAINER_EMAIL");
            license = var("PKG_FLAKE_LICENSE");
        );
        
        // Read from stdin where still not given
        if !self.ci {
            fill_in!(options:
                name = user_input("Name");
                description = user_input("Description");
                version = user_input("Version");
                url = user_input("URL");
                maintainer_name = user_input("Maintainer Name");
                maintainer_email = user_input("Maintainer Email");
                license = user_input("License");
            );
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

#[derive(Debug, Default, Args, Clone, Deserialize)]
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
    pub fn update(self, other: Self) -> Self {
        Self {
            name: other.name.or(self.name),
            description: other.description.or(self.description),
            version: other.version.or(self.version),
            url: other.url.or(self.url),
            maintainer_name: other.maintainer_name.or(self.maintainer_name),
            maintainer_email: other.maintainer_email.or(self.maintainer_email),
            license: other.license.or(self.license),
        }
    }

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
