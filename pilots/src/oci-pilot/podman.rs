use std::{io::Error, path::PathBuf, process::Command};

use flakes::config::{itf::FlakeConfig, CID_DIR};

/// Data Sync
///
struct DataSync {}
impl DataSync {
    /// Sync static data
    fn sync_static(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Sync layers
    fn sync_delta(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Sync files/dirs specified in target/defaults::HOST_DEPENDENCIES
    /// from the running host to the target path
    fn sync_host(&self) -> Result<(), Error> {
        Ok(())
    }

    // Prune an image by URI
    fn prune(&self, uri: &str) -> Result<(), Error> {
        Ok(())
    }

    /// Initialise environment
    fn get_cid_dir(&self) -> Result<PathBuf, Error> {
        Ok(PathBuf::from(""))
    }

    /// Flush a cid file
    fn flush_cid(&self) -> Result<(), Error> {
        // Probably needs an enum with a different errors
        Ok(())
    }

    /// Flush all cids
    fn flush_all_cids(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[derive(Default)]
struct DataTracker {
    data: Vec<String>,
}

impl DataTracker {
    /// Add data
    fn add(&mut self, item: String) -> &mut Self {
        self.data.push(item);
        self
    }

    /// Dump data to a file
    fn dump(&self, dst: PathBuf) -> Result<(), Error> {
        Ok(())
    }
}

pub(crate) struct PodmanRunner {
    app: String,
    cfg: FlakeConfig,
}

impl PodmanRunner {
    pub(crate) fn new(app: String, cfg: FlakeConfig) -> Self {
        PodmanRunner { app, cfg }
    }

    /// Create a CID file
    pub(crate) fn create_cid(&self) -> PathBuf {
        let mut suff = String::from("");
        for arg in std::env::args().collect::<Vec<String>>() {
            if arg.starts_with('@') {
                suff = format!("-{}", arg.to_owned());
                break;
            }
        }

        CID_DIR.join(format!("{}{}.cid", self.app.to_owned(), suff))
    }

    /// Get config
    fn get_cfg(&self) -> &FlakeConfig {
        &self.cfg
    }

    fn call(&self, args: &[&str]) -> Result<String, Error> {
        let mut cmd = Command::new("sudo");
        if let Some(user) = self.get_cfg().runtime().run_as() {
            cmd.arg("--user").arg(user.name).arg("podman");
        } else {
            cmd = Command::new("podman");
        }

        for arg in args {
            cmd.arg(arg);
        }

        match cmd.output() {
            Ok(out) => return Ok(String::from_utf8(out.stdout).unwrap_or_default()),
            Err(out) => return Err(Error::from(out)),
        }
    }

    /// Create a container
    fn create_container(&self) -> Result<(String, String), Error> {
        Ok(("".to_string(), "".to_string()))
    }
}

/// Podman runtime
///
pub(crate) struct PodmanPilot {
    appdir: PathBuf,
    runner: PodmanRunner,
}

impl PodmanPilot {
    /// Constructor of a new Podman Pilot instance
    pub(crate) fn new() -> Result<Self, Error> {
        let appdir = flakes::config::app_path()?;
        Ok(PodmanPilot {
            appdir: appdir.to_owned(),
            runner: PodmanRunner::new(appdir.file_name().unwrap().to_str().unwrap().to_string(), flakes::config::load()?),
        })
    }

    /// Start Podman Pilot instance
    pub(crate) fn start() -> Result<(), Error> {
        Ok(())
    }

    /// Returns true if a container is running
    fn is_running(&self) -> Result<bool, Error> {
        Ok(false)
    }

    fn is_image_exists(&self, name: &str) -> Result<bool, Error> {
        Ok(false)
    }

    /// Find container by the ID and call an action there
    fn call_instance(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Get relevant exported path
    fn get_exported_path(&self) -> PathBuf {
        PathBuf::from("")
    }

    /// Mount container. Returns the mount point, if succeeded
    fn mount(&self, as_image: bool) -> Result<PathBuf, Error> {
        Ok(PathBuf::from("/mount/point"))
    }

    /// Umount container.
    fn umount(&self, mountpoint: PathBuf, as_image: bool) -> Result<(), Error> {
        Ok(())
    }
}
