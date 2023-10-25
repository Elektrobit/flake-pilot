pub mod builder;
mod options;

use std::{
    env,
    fmt::Display,
    process::{Command, ExitCode, ExitStatus},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use sys_info::LinuxOSReleaseInfo;

const NO_PACMAN_FOUND: &str = "No native package manager found. You can try to run the build tools directly. See `flake-ctl help` for a list of available tools";

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();

    match cli.subcmd.unwrap_or_default() {
        Subcmds::About => {
            println!("Package flakes and images with the native package manager;TOOL");
            Ok(ExitCode::SUCCESS)
        }
        cmd @ (Subcmds::Which | Subcmds::Build) => {
            let pacman = PackageManager::try_find_local_manager().context(NO_PACMAN_FOUND)?;

            if cmd == Subcmds::Which {
                println!("{pacman}");
                Ok(ExitCode::SUCCESS)
            } else {
                match pacman.run()?.code() {
                    Some(code) => Ok((code as u8).into()),
                    None => Ok(ExitCode::FAILURE),
                }
            }
        }
    }
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
enum Subcmds {
    #[clap(hide = true)]
    About,
    /// Print the name of the native packaging tool
    Which,
    #[clap(hide = true)]
    Build,
}

impl Default for Subcmds {
    fn default() -> Self {
        Self::Build
    }
}

#[derive(Parser)]
/// Package a flake using your native packaging tool
struct Cli {
    #[clap(subcommand)]
    subcmd: Option<Subcmds>,
    #[clap(trailing_var_arg = true)]
    args: Vec<String>,
}

#[derive(Debug)]
enum PackageManager {
    Rpm,
    Dpkg,
}

impl Display for PackageManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageManager::Rpm => f.write_str("rpmbuild"),
            PackageManager::Dpkg => f.write_str("dpkg-deb"),
        }
    }
}

impl PackageManager {
    fn try_find_local_manager() -> Result<Self> {
        let LinuxOSReleaseInfo { id, id_like, .. } = sys_info::linux_os_release().context("Failed to read /etc/os-release")?;
        id_like
            .into_iter()
            .flat_map(|list| list.split(' ').map(str::to_owned).collect::<Vec<_>>())
            .chain(id.into_iter())
            .map(|x| x.parse())
            .find(Result::is_ok)
            .unwrap_or(Err(anyhow!("No Packagemanager available")))
    }

    fn run(&self) -> Result<ExitStatus> {
        Command::new(match self {
            PackageManager::Rpm => "flake-ctl-build-rpmbuild",
            PackageManager::Dpkg => "flake-ctl-build-dpkg",
        })
        .args(env::args().skip(1))
        .status()
        .context("Failed to run builder")
    }
}

impl FromStr for PackageManager {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "debian" => Ok(Self::Dpkg),
            "redhat" => Ok(Self::Rpm),
            _ => Err(anyhow!("No Packagemanager available")),
        }
    }
}
