mod build;
pub mod init;

use std::{fs::{create_dir, remove_dir, remove_dir_all}, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use flake_ctl_build::PackageOptionsBuilder;

fn main() -> Result<()> {
    Cli::parse().run()
}

#[derive(Parser)]
enum Cli {
    New {
        app: PathBuf,
        /// Ignore global options in ~/.flakes/package/options.yaml
        #[clap(long)]
        scratch: bool,
        #[clap(flatten)]
        options: PackageOptionsBuilder,
    },
    Init {
        app: PathBuf,
        /// Ignore global options in ~/.flakes/package/options.yaml
        #[clap(long)]
        scratch: bool,
        #[clap(flatten)]
        options: PackageOptionsBuilder,
    },
    Build {},
    Clean {},
}

impl Cli {
    fn run(self) -> Result<()> {
        match self {
            Self::New { options, scratch, app } => init::new(options, scratch, &app),
            Self::Init { options, scratch, app } => init::init(options, scratch, &app),
            Self::Build {} => build::build(),
            Self::Clean {} => remove_dir_all("out").context("No output directory to remove"),
        }
    }
}
