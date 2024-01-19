use crate::paths::flake_dir_from;

use self::{cfgparse::FlakeCfgParser, itf::FlakeConfig};
use lazy_static::lazy_static;
use std::{
    env, fs,
    io::Error,
    path::{Path, PathBuf},
    sync::Mutex,
};

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
    static ref _CID_DIR: PathBuf = PathBuf::from("/usr/share/flakes/cid");

    /// CID directory where all OCI containers should store their runtime ID, but in the user's home
    static ref _CID_HDIR: PathBuf = PathBuf::from(".flakes");

    // Global internal variable to keep singleton content for the CID directory, taken by `get_cid_store` function.
    static ref CID_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

    // Config instance
    static ref CFG: FlakeConfig = load().unwrap();

    /// Local Config directory for flake packaging
    pub static ref LOCAL_PACKAGING_CONFIG: PathBuf = PathBuf::from(".flakes/package/options.yaml");

    /// Global Config directory for flake packaging
    pub static ref GLOBAL_PACKAGING_CONFIG: PathBuf = home::home_dir().unwrap().join(&*LOCAL_PACKAGING_CONFIG);
}

/// Get CID store, depending on the call
pub fn get_cid_store() -> Result<PathBuf, Error> {
    let mut cid_dir_val = CID_DIR.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(cid_dir) = cid_dir_val.clone() {
        log::debug!("Return existing CID store at {:?}", cid_dir);
        return Ok(cid_dir);
    }

    let cid_dir = home::home_dir()
        .map(|hd| hd.join(_CID_HDIR.as_path()))
        .unwrap_or(_CID_DIR.to_path_buf())
        .join("cid");

    if !cid_dir.exists() {
        match fs::create_dir(&cid_dir) {
            Ok(_) => {}
            Err(err) => {
                return Err(Error::new(std::io::ErrorKind::NotFound, format!("Unable create CID directory: {}", err)));
            }
        }
    }

    if std::fs::metadata(&cid_dir).unwrap().permissions().readonly() {
        return Err(Error::new(
            std::io::ErrorKind::PermissionDenied,
            format!("Unable to write to \"{}\" directory", cid_dir.as_os_str().to_str().unwrap()),
        ));
    }

    *cid_dir_val = Some(cid_dir.clone());


    log::debug!("Return new CID store instance at {:?}", cid_dir);
    Ok(cid_dir)
}

/// Find path on itself
pub fn app_path() -> Result<PathBuf, Error> {
    let loc = env::args().next().unwrap();
    if loc.starts_with('/') {
        // Absolute
        return Ok(PathBuf::from(loc));
    } else if loc.contains('/') {
        // Relative
        let rel = PathBuf::from(loc);
        if let Ok(loc) = env::current_dir() {
            return Ok(path_clean::clean(loc.join(rel)));
        }
    } else {
        // Needs resolve
        if let Ok(loc) = which::which(PathBuf::from(loc).file_name().unwrap().to_str().unwrap()) {
            return Ok(loc);
        }
    }

    env::current_exe()
}

pub fn get() -> FlakeConfig {
    CFG.to_owned()
}

pub fn load_from_path(path: &Path) -> Result<FlakeConfig, Error> {
    // Get app configuration

    let cfg_d_paths: Vec<PathBuf> = std::fs::read_dir(path.with_extension("d"))
        .ok()
        .into_iter()
        .flatten()
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

    match FlakeCfgParser::new(path.with_extension("yaml"), cfg_d_paths)?.parse() {
        Some(cfg) => Ok(cfg),
        None => Err(Error::new(std::io::ErrorKind::NotFound, "Unable to read configuration")),
    }
}

pub fn load_from_target(root: Option<&Path>, app_p: &Path) -> Result<FlakeConfig, Error> {
    let app_ps = app_p.file_name().unwrap().to_str().unwrap().to_string();
    load_from_path(&flake_dir_from(root).join(app_ps))
}

/// Load config for the host app path
pub fn load() -> Result<FlakeConfig, Error> {
    //pub fn load_for_app() {
    let app_p = app_path().unwrap();
    load_from_target(None, &app_p)
}
