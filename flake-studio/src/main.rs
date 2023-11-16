mod build;
pub mod init;

use std::{fs::remove_dir_all, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use flake_ctl_build::PackageOptionsBuilder;
use init::Init;

fn main() -> Result<()> {
    Cli::parse().run()
}

#[derive(Parser)]
enum Cli {
    New {
        name: String,
        #[clap(flatten)]
        init: Init
    },
    Init{
        #[clap(flatten)]
        init: Init
    },
    Build {},
    Clean {},
}

impl Cli {
    fn run(self) -> Result<()> {
        match self {
            Self::New { name, init} => init::new(name, init),
            Self::Init { init } => init::init(init),
            Self::Build {} => build::build(),
            Self::Clean {} => remove_dir_all("out").context("No output directory to remove"),
        }
    }
}
