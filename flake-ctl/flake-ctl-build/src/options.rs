use std::{
    env::var,
    io::{stdin, stdout, Write},
};

use crate::config::{get_global, get_local};
use anyhow::{anyhow, Context, Result};
use clap::Args;
use colored::Colorize;
use serde::{Deserialize, Serialize};

fn user_input(name: &str) -> Option<String> {
    let mut buf = String::new();
    print!("{}: ", name.bold());
    stdout().flush().ok();
    stdin().read_line(&mut buf).ok();
    let buf = buf.trim_end().to_owned();
    print!("{}{}\r", termion::cursor::Up(1), termion::clear::CurrentLine);
    stdout().flush().ok();
    if buf.is_empty() {
        None
    } else {
        Some(buf)
    }
}

macro_rules! fill_in {
    ($options:ident : $($name:ident = $getter:expr;)*) => {
        $($options.$name = $options.$name.or_else(|| $getter);)*
    };
}

pub fn determine_options(matches: &clap::ArgMatches) -> Result<PackageOptions> {
    let mut options = PackageOptionsBuilder::default();

    // Get options from global/local settings
    if let Ok(global) = get_global() {
        options = options.update(global);
    }
    if let Ok(local) = get_local() {
        options = options.update(local);
    }

    // Options on CLI override global/local settings
    // options = options.update(self.options.clone());

    fill_in!(options:
        name = matches.get_one("name").cloned();
        description = matches.get_one("description").cloned();
        version = matches.get_one("version").cloned();
        url = matches.get_one("url").cloned();
        maintainer_name = matches.get_one("maintainer_name").cloned();
        maintainer_email = matches.get_one("maintainer_email").cloned();
        license = matches.get_one("license").cloned();
    );

    // Read from env where not given
    fill_in!(options:
        name = var("PKG_FLAKE_NAME").ok();
        description = var("PKG_FLAKE_DESCRIPTION").ok();
        version = var("PKG_FLAKE_VERSION").ok();
        url = var("PKG_FLAKE_URL").ok();
        maintainer_name = var("PKG_FLAKE_MAINTAINER_NAME").ok();
        maintainer_email = var("PKG_FLAKE_MAINTAINER_EMAIL").ok();
        license = var("PKG_FLAKE_LICENSE").ok();
    );

    // Read from stdin where still not given
    if !matches.get_flag("ci") {
        options = options.fill_from_terminal();
    }

    options.build().context("Missing packaging option")
}

#[derive(Debug, Clone)]
pub struct PackageOptions {
    pub name: String,
    pub description: String,
    pub version: String,
    pub url: String,
    pub maintainer_name: String,
    pub maintainer_email: String,
    pub license: String,
}

#[derive(Debug, Default, Args, Clone, Serialize, Deserialize)]
pub struct PackageOptionsBuilder {
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// The name of the package (excluding version, arch, etc.)
    pub name: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    /// A url pointing to the packages source
    pub url: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer_name: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maintainer_email: Option<String>,
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
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

    pub fn fill_from_terminal(mut self) -> Self {
        fill_in!(self:
            name = user_input("Name");
            description = user_input("Description");
            version = user_input("Version");
            url = user_input("URL");
            maintainer_name = user_input("Maintainer Name");
            maintainer_email = user_input("Maintainer Email");
            license = user_input("License");
        );
        self
    }
}
