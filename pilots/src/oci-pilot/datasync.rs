use flakes::config::CID_DIR;
use std::{io::Error, path::PathBuf};

/// Data Sync
///
pub(crate) struct DataSync {}

impl DataSync {
    /// Sync static data
    pub(crate) fn sync_static(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Sync layers
    pub(crate) fn sync_delta(&self) -> Result<(), Error> {
        Ok(())
    }

    /// Sync files/dirs specified in target/defaults::HOST_DEPENDENCIES
    /// from the running host to the target path
    pub(crate) fn sync_host(&self) -> Result<(), Error> {
        Ok(())
    }

    // Prune an image by URI
    pub(crate) fn prune(&self, uri: &str) -> Result<(), Error> {
        Ok(())
    }

    /// Initialise environment
    pub(crate) fn check_cid_dir(&self) -> Result<PathBuf, Error> {
        if !CID_DIR.exists() {
            return Err(Error::new(
                std::io::ErrorKind::NotFound,
                format!("CID directory \"{}\" was not found", CID_DIR.as_os_str().to_str().unwrap()),
            ));
        }

        if std::fs::metadata(CID_DIR.to_path_buf()).unwrap().permissions().readonly() {
            return Err(Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Unable to write to \"{}\" directory", CID_DIR.as_os_str().to_str().unwrap()),
            ));
        }

        Ok(CID_DIR.to_owned())
    }

    /// Flush a cid file
    pub(crate) fn flush_cid(&self) -> Result<(), Error> {
        // Probably needs an enum with a different errors
        Ok(())
    }

    /// Flush all cids
    pub(crate) fn flush_all_cids(&self) -> Result<(), Error> {
        Ok(())
    }
}
