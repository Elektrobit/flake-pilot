use anyhow::Result;
use clap::Parser;
use flake_ctl_build::{self, FlakeBuilder, Path, PathBuf};

mod debbuild;

fn main() -> Result<()> {
    let args = Args::parse();
    let template = args.template.unwrap_or_else(|| Path::new("/usr/share/flakes/package/debbuild").to_owned());
    debbuild::Builder { template_dir: &template, edit: !args.no_edit }.run(&args.builder_args)?;
    Ok(())
}

#[derive(Parser)]
struct Args {
    #[command(flatten)]
    builder_args: flake_ctl_build::BuilderArgs,

    #[arg(long)]
    template: Option<PathBuf>,

    #[arg(long)]
    no_edit: bool,
}
