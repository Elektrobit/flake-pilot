use std::env;

pub const CONTAINER_FLAKE_DIR: &str = "/usr/share/flakes";
pub const CONTAINER_DIR: &str = "/var/lib/containers";
pub const CONTAINER_CID_DIR: &str = "/var/lib/containers/storage/tmp/flakes";
pub const GC_THRESHOLD: i32 = 20;
pub const HOST_DEPENDENCIES: &str = "removed";

pub fn debug(message: &str) {
    if env::var("PILOT_DEBUG").is_ok() {
        debug!("{}", message)
    };
}
