use anyhow::Result;
use clap::Parser;
use rpmbuild::Tooling;
use flake_ctl_build::{self, FlakeBuilder, Path, PathBuf};

mod rpmbuild;

fn main() -> Result<()> {
    let args = Args::parse();
    let template = args.template.unwrap_or_else(|| Path::new("/usr/share/flakes/package/debbuild").to_owned());
    rpmbuild::Builder { template_dir: &template, edit: !args.no_edit, tooling: args.tooling }.run(&args.builder_args)?;
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

    #[arg(long, value_enum, default_value = Tooling::RPMBuild)]
    tooling: Tooling,
}
