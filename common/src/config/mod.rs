use self::{cfgparse::FlakeCfgParser, itf::FlakeConfig};
use lazy_static::lazy_static;
use std::{env, fs, io::Error, path::PathBuf, sync::Mutex};

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
    pub static ref _CID_DIR: String = "/usr/share/flakes/cid".to_string();

    /// CID directory where all OCI containers should store their runtime ID, but in the user's home
    pub static ref _CID_HDIR: String = ".flakes".to_string();

    pub static ref CFG: FlakeConfig = load().unwrap();

    // Global internal variable to keep singleton content for the CID directory, taken by `get_cid_store` function.
    static ref CID_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
}

/// Get CID store, depending on the call
pub fn get_cid_store() -> Result<PathBuf, Error> {
    let mut cid_dir_val = CID_DIR.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(cid_dir) = cid_dir_val.clone() {
        log::debug!("Return existing CID store at {:?}", cid_dir);
        return Ok(cid_dir);
    }

    let mut cid_dir: PathBuf = PathBuf::from("");
    if let Some(hd) = env::var_os("HOME") {
        let homedir = hd.as_os_str().to_str().unwrap_or("").to_string();
        if !homedir.is_empty() {
            cid_dir = PathBuf::from(homedir).join(_CID_HDIR.to_string());
        }
    } else {
        cid_dir = PathBuf::from(_CID_DIR.to_string());
    }

    if !cid_dir.exists() {
        match fs::create_dir(cid_dir.to_owned()) {
            Ok(_) => {}
            Err(err) => {
                return Err(Error::new(std::io::ErrorKind::NotFound, format!("Unable create CID directory: {}", err)));
            }
        }
    }

    if std::fs::metadata(cid_dir.to_path_buf()).unwrap().permissions().readonly() {
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

/// Load config for the host app path
fn load() -> Result<FlakeConfig, Error> {
    //pub fn load_for_app() {
    let app_p = app_path().unwrap();
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
