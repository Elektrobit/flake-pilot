use std::{
    env,
    fmt::Display,
    process::{Command, ExitCode, ExitStatus},
    str::FromStr,
};

use anyhow::{anyhow, Context, Result};
use sys_info::LinuxOSReleaseInfo;

const NO_PACMAN_FOUND: &str = "No native package manager found. You can try to run the build tools directly. See `flake-ctl help` for a list of available tools";

use anyhow::Ok;

fn main() -> Result<ExitCode> {
    match env::args().nth(1).as_deref() {
        Some("about") => {
            println!("Package flakes and images with your systems native package manager;PACKAGER");
            Ok(ExitCode::SUCCESS)
        }
        Some("which") => {
            let pacman = PackageManager::try_find_local_manager().context(NO_PACMAN_FOUND)?;
            println!("{pacman};{}", pacman.builder());
            Ok(ExitCode::SUCCESS)
        }
        _ => {
            let pacman = PackageManager::try_find_local_manager().context(NO_PACMAN_FOUND)?;
            match pacman.run()?.code() {
                Some(code) => Ok((code as u8).into()),
                None => Ok(ExitCode::FAILURE),
            }
        }
    }
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
            PackageManager::Dpkg => f.write_str("dpkg-buildpackage"),
        }
    }
}

impl PackageManager {
    fn try_find_local_manager() -> Result<Self> {
        let LinuxOSReleaseInfo { id, id_like, .. } = sys_info::linux_os_release().context("Failed to read /etc/os-release")?;
        id_like
            .into_iter()
            .flat_map(|list| list.split(' ').map(str::to_owned).collect::<Vec<_>>())
            .chain(id)
            .map(|x| x.parse())
            .find(Result::is_ok)
            .unwrap_or(Err(anyhow!("No Packagemanager available")))
    }

    fn builder(&self) -> &str {
        match self {
            PackageManager::Rpm => "flake-ctl-build-rpmbuild",
            PackageManager::Dpkg => "flake-ctl-build-dpkg",
        }
    }

    fn run(&self) -> Result<ExitStatus> {
        Command::new(self.builder()).args(env::args().skip(1)).status().context("Failed to run builder")
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
