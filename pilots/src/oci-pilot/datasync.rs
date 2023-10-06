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
