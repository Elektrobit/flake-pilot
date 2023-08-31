use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum MountMode {
    Bind,
    // Devpts,
    // Glob,
    // Image,
    // Tmpfs,
    // Volume,
}

pub fn format_mount_command<Source: AsRef<Path>, Target: AsRef<Path>>(source: Source, target: Target, mode: &MountMode) -> String {

    let source = source.as_ref().to_string_lossy();
    let target = target.as_ref().to_string_lossy();

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
