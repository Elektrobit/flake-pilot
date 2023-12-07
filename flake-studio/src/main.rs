mod build;
pub mod init;
pub mod util;
pub mod common;

use std::fs::remove_dir_all;

use anyhow::{Context, Result};
use clap::Parser;
use init::InitArgs;
use util::discover_project_root;

fn main() -> Result<()> {
    Cli::parse().run()
}

#[derive(Parser)]
enum Cli {
    New {
        project_name: String,
        #[clap(flatten)]
        init: InitArgs,
    },
    Init {
        #[clap(flatten)]
        boba: InitArgs,
    },
    Build {
        #[clap(long)]
        keep: bool
    },
    Clean {},
}

impl Cli {
    fn run(self) -> Result<()> {
        match self {
            Self::New { project_name: name, init } => init::new(name, init),
            Self::Init { boba, .. } => init::init(boba),
            Self::Build { keep } => build::build(keep),
            Self::Clean {} => {
                discover_project_root()?;
                remove_dir_all("out").context("No output directory to remove")
            }
        }
    }
}
