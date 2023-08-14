use std::path::Path;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub enum MountMode {
    Bind,
    // Devpts,
    // Glob,
    // Image,
    // Tmpfs,
    // Volume,
}

pub fn mount_working_directory<Source: AsRef<Path>, Target: AsRef<Path>>(source: Source, target: Target, mode: &MountMode) -> String {

    let source = source.as_ref().to_str().unwrap_or_default();
    let target = target.as_ref().to_str().unwrap_or_default();

    match mode {
        MountMode::Bind => format!("type=bind,source={source},destination={target}"),
        // MountMode::Devpts => unimplemented!(),
        // MountMode::Glob => unimplemented!(),
        // MountMode::Image => unimplemented!(),
        // MountMode::Tmpfs => unimplemented!(),
        // MountMode::Volume => unimplemented!(),
    }

}

impl Default for MountMode {
    fn default() -> Self {
        Self::Bind
    }
}
