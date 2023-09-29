use std::{io::Error, path::PathBuf};

#[derive(Default)]
pub(crate) struct DataTracker {
    data: Vec<String>,
}

impl DataTracker {
    /// Add data
    pub(crate) fn add(&mut self, item: String) -> &mut Self {
        self.data.push(item);
        self
    }

    /// Dump data to a file
    pub(crate) fn dump(&self, dst: PathBuf) -> Result<(), Error> {
        Ok(())
    }
}
