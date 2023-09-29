use crate::datasync::DataSync;
use flakes::config::itf::InstanceMode;
use flakes::config::{itf::FlakeConfig, CID_DIR};
use std::{fs, io::Error, path::PathBuf, process::Command, vec};

pub(crate) struct PodmanRunner {
    datasync: DataSync,
    app: String,
    cfg: FlakeConfig,
}

impl PodmanRunner {
    pub(crate) fn new(app: String, cfg: FlakeConfig) -> Self {
        PodmanRunner { datasync: DataSync {}, app, cfg }
    }

    /// Make a CID file
    pub(crate) fn get_cid(&self) -> PathBuf {
        let mut suff = String::from("");
        for arg in std::env::args().collect::<Vec<String>>() {
            if arg.starts_with('@') {
                suff = format!("-{}", &arg.to_owned()[1..]);
                break;
            }
        }

        CID_DIR.join(format!("{}{}.cid", self.app.to_owned(), suff))
    }

    /// Garbage collect CID.
    ///
    /// Check if container exists according to the specified
    /// container_cid_file. Garbage cleanup the container_cid_file
    /// if no longer present. Return a true value if the container
    /// exists, in any other case return false.
    pub(crate) fn gc_cid(&self, cid: PathBuf) -> Result<bool, Error> {
        if !cid.exists() {
            return Ok(true);
        }

        match self.call(false, &["container", "exists", &fs::read_to_string(&cid)?]) {
            Ok(_) => Ok(true),
            Err(_) => {
                fs::remove_file(cid)?;
                Ok(false)
            }
        }
    }

    /// Get config
    fn get_cfg(&self) -> &FlakeConfig {
        &self.cfg
    }

    fn call(&self, output: bool, args: &[&str]) -> Result<String, Error> {
        let mut cmd = Command::new("sudo");
        if let Some(user) = self.get_cfg().runtime().run_as() {
            cmd.arg("--user").arg(user.name).arg("podman");
        } else {
            cmd = Command::new("podman");
        }

        for arg in args {
            cmd.arg(arg);
        }

        if output {
            match cmd.output() {
                Ok(out) => {
                    return Ok(String::from_utf8(out.stdout).unwrap_or_default());
                }
                Err(out) => {
                    return Err(out);
                }
            }
        } else {
            match cmd.status() {
                Ok(st) => {
                    if !st.success() {
                        return Err(Error::new(std::io::ErrorKind::InvalidData, "Call error"));
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok("".to_string())
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
            runner: PodmanRunner::new(appdir.file_name().unwrap().to_str().unwrap().to_string(), flakes::config::get()),
        })
    }

    /// Start Podman Pilot instance
    pub(crate) fn start(&self) -> Result<(), Error> {
        self.runner.create_container();
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
