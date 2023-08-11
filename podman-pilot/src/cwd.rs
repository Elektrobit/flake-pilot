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

pub fn mount_working_directory<T: AsRef<Path>>(dir: T, mode: &MountMode) -> String {

    let path = dir.as_ref().to_str().unwrap_or_default();
    match mode {
        MountMode::Bind => format!("type=bind,source={path},destination={path}"),
        // MountMode::Devpts => unimplemented!(),
        // MountMode::Glob => unimplemented!(),
        // MountMode::Image => unimplemented!(),
        // MountMode::Tmpfs => unimplemented!(),
        // MountMode::Volume => unimplemented!(),
    }

}
