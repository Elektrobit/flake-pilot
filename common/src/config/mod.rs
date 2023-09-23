use self::{cfgparse::FlakeCfgParser, itf::FlakeConfig};
use lazy_static::lazy_static;
use std::{env, io::Error, path::PathBuf};

pub mod cfg_v1;
pub mod cfg_v2;
pub mod cfgparse;
pub mod itf;
pub mod pilots;

lazy_static! {
    /// Flake directory for all the app configurations and other shared data
    pub static ref FLAKE_DIR: PathBuf = PathBuf::from("/usr/share/flakes");

    /// Default OCI container directory for storage etc
    pub static ref DEFAULT_CONTAINER_DIR: PathBuf = PathBuf::from("/var/lib/containers");

    /// CID directory where all OCI containers should store their runtime ID
    pub static ref CID_DIR: PathBuf = PathBuf::from("/usr/share/flakes/cid");
}

/// Find path on itself
fn path_for_app() -> Result<PathBuf, Error> {
    let loc = env::args().next().unwrap();
    if loc.starts_with("/") {
        // Absolute
        return Ok(PathBuf::from(loc));
    } else if loc.contains("/") {
        // Relative
        let rel = PathBuf::from(loc);
        if let Ok(loc) = env::current_dir() {
            return Ok(path_clean::clean(loc.join(rel)));
        }
    } else {
        // Needs resolve
        if let Ok(loc) = which::which(PathBuf::from(loc).file_name().unwrap().to_str().unwrap().to_string()) {
            return Ok(loc);
        }
    }

    env::current_exe()
}

/// Load config for the host app path
pub fn load() -> Result<FlakeConfig, Error> {
    //pub fn load_for_app() {
    let app_p = path_for_app().unwrap();
    let app_ps = app_p.file_name().unwrap().to_str().unwrap().to_string();

    // Get app configuration
    let cfg_d_paths: Vec<PathBuf> = std::fs::read_dir(FLAKE_DIR.join(app_ps.to_owned() + ".d"))
        .unwrap()
        .filter_map(|entry| -> Option<PathBuf> {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    if let Some(file_name) = entry.file_name().to_str() {
                        return Some(PathBuf::from(file_name.to_string()));
                    }
                }
            }
            None
        })
        .collect();

    match FlakeCfgParser::new(FLAKE_DIR.join(app_ps + ".yaml"), cfg_d_paths)?.parse() {
        Some(cfg) => Ok(cfg),
        None => Err(Error::new(std::io::ErrorKind::NotFound, "Unable to read configuration")),
    }
}
