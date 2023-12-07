use std::{process::Command, time::UNIX_EPOCH};

use anyhow::{bail, Result};
use base64::{engine::general_purpose, Engine as _};

pub fn setup_flake(app: &str, image: &str) -> Result<()> {
    let out = Command::new("flake-ctl-build")
        .args(["image", "podman"])
        .arg(image)
        .arg(format!("/usr/bin/{app}"))
        .args(["--location", ".staging", "--ci", "--keep", "--dry-run"])
        .output()?;
    if !out.status.success() {
        bail!("{}", String::from_utf8_lossy(&out.stderr))
    }
    Ok(())
}

/// Constructs a temporary image name by including the current timestamp (in base64 and lowercased).
///
/// Format: `{app_name}.flake.{timestamp}`
///
/// This name should not be relied upon except for the purpose of making image names
/// quasi-unique. Since it is lowercased it can not be decoded back into a valid timestamp.
pub fn image_name(app_name: &str) -> String {
    let tail = general_purpose::STANDARD_NO_PAD
        .encode(format!("{}", std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()))
        .to_ascii_lowercase();
    format!("{app_name}.flake.{tail}")
}
