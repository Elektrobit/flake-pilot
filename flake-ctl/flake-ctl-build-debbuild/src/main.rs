use anyhow::Result;
use clap::Parser;
use flake_ctl_build::{self, FlakeBuilder};

mod debbuild;

fn main() -> Result<()> {
    let args = Args::parse();

    let options = args.builder_args.determine_options()?;
    debbuild::Builder.execute(&options, args.builder_args.target, args.builder_args.location, true, false)?;
    Ok(())
}

#[derive(Parser)]
struct Args {
    #[command(flatten)]
    builder_args: flake_ctl_build::BuilderArgs,
}
