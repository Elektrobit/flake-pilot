use std::{io::Error, path::PathBuf};

use flakes::config::itf::FlakeConfig;

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

/// Podman runtime
///
pub(crate) struct PodmanPilot {
    cfg: FlakeConfig,
}

impl PodmanPilot {
    /// Constructor of a new Podman Pilot instance
    pub(crate) fn new() -> Result<Self, Error> {
        Ok(PodmanPilot { cfg: flakes::config::load()? })
    }

    /// Start Podman Pilot instance
    pub(crate) fn start() -> Result<(), Error> {
        let appdir = flakes::config::app_path()?;
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
