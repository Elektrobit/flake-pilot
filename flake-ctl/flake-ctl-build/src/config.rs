use std::{fs::OpenOptions, path::Path};

use anyhow::{Context, Result};
use flakes::config::{GLOBAL_PACKAGING_CONFIG, LOCAL_PACKAGING_CONFIG};

use crate::options::PackageOptionsBuilder;

pub fn get_global() -> Result<PackageOptionsBuilder> {
    get_from(&*GLOBAL_PACKAGING_CONFIG)
}

pub fn get_local() -> Result<PackageOptionsBuilder> {
    get_from(&*LOCAL_PACKAGING_CONFIG)
}

fn get_from(x: impl AsRef<Path>) -> Result<PackageOptionsBuilder> {
    let path = x.as_ref();
    let reader = OpenOptions::new().read(true).open(&path);
    let path = path.to_string_lossy();
    let reader = reader.context(format!("Could not open {path}"))?;
    serde_yaml::from_reader(reader).context(format!("Failed to deserialize {path}"))
}
