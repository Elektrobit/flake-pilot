//
// Copyright (c) 2022 Elektrobit Automotive GmbH
//
// This file is part of flake-pilot
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
use anyhow::{bail, Context, Result};
use flakes::config::{load_from_target, FLAKE_DIR};
use flakes::paths::PathExt;
use log::{error, info, warn};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use tempfile::tempdir;

use crate::defaults;
use crate::{app, app_config};

pub fn pull(uri: &String) -> i32 {
    /*!
    Call podman pull and prune with the provided uri
    !*/
    let mut status_code = 255;

    info!("Fetching from registry...");
    info!("podman pull {}", uri);
    let status = Command::new(defaults::PODMAN_PATH).arg("pull").arg(uri).status();

    match status {
        Ok(status) => {
            status_code = status.code().unwrap();
            if !status.success() {
                error!("Failed, error message(s) reported");
            } else {
                info!("podman prune");
                let _ = Command::new(defaults::PODMAN_PATH).arg("image").arg("prune").arg("--force").status();
            }
        }
        Err(status) => {
            error!("Process terminated by signal: {}", status)
        }
    }

    status_code
}

pub fn load(oci: &String) -> i32 {
    /*!
    Call podman load with the provided oci tar file
    !*/
    let mut status_code = 255;

    info!("Loading OCI image...");
    info!("podman load -i {}", oci);
    let status = Command::new(defaults::PODMAN_PATH).arg("load").arg("-i").arg(oci).status();

    match status {
        Ok(status) => {
            status_code = status.code().unwrap();
            if !status.success() {
                error!("Failed, error message(s) reported");
            }
        }
        Err(status) => {
            error!("Process terminated by signal: {}", status)
        }
    }

    status_code
}

pub fn rm(container: &String) {
    /*!
    Call podman image rm with force option to remove all running containers
    !*/
    info!("Removing image and all running containers...");
    info!("podman rm -f  {}", container);
    let status = Command::new(defaults::PODMAN_PATH).arg("image").arg("rm").arg("-f").arg(container).status();

    match status {
        Ok(status) => {
            if !status.success() {
                error!("Failed, error message(s) reported");
            }
        }
        Err(status) => {
            error!("Process terminated by signal: {}", status)
        }
    }
}

pub fn mount_container(container_name: &str) -> String {
    /*!
    Mount container and return mount point,
    or an empty string in the error case
    !*/
    match Command::new(defaults::PODMAN_PATH).arg("image").arg("mount").arg(container_name).output() {
        Ok(output) => {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).strip_suffix('\n').unwrap().to_string();
            }
            error!("Failed to mount container image: {}", String::from_utf8_lossy(&output.stderr));
        }
        Err(error) => {
            error!("Failed to execute podman image mount: {:?}", error)
        }
    }
    "".to_string()
}

pub fn umount_container(container_name: &str) -> i32 {
    /*!
    Umount container image
    !*/
    let mut status_code = 255;
    match Command::new(defaults::PODMAN_PATH)
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .arg("image")
        .arg("umount")
        .arg(container_name)
        .status()
    {
        Ok(status) => {
            status_code = status.code().unwrap();
        }
        Err(error) => {
            error!("Failed to execute podman image umount: {:?}", error)
        }
    }
    status_code
}

pub fn purge_container(root: &Path, container: &str) -> Result<()> {
    /*!
    Iterate over all yaml config files and find those connected
    to the container. Delete all app registrations for this
    container and also delete the container from the local
    registry
    !*/
    for app_name in app::app_names() {
        let config_file = Path::new(defaults::FLAKE_DIR).join(app_name).with_extension("yaml");
        match app_config::AppConfig::from_file(&config_file) {
            Ok(app_conf) if app_conf.container.name == container => {
                let path = &app_conf.container.host_app_path;
                app::remove(root, Path::new(path)).map_err(|err| warn!("Could not delete {path}: {err}")).ok();
            }
            Ok(_) => (),
            Err(error) => warn!("Error in flake \"{}\": {}", config_file.to_string_lossy(), error),
        };
    }
    rm(&container.to_string());
    Ok(())
}

pub fn print_container_info(container: &str) -> Result<()> {
    /*!
    Print app info file

    Lookup container_base_name.yaml file in the root of the
    specified container and print the file if it is present
    !*/
    let container_basename = Path::new(container).file_name().unwrap().to_str().unwrap();
    let image_mount_point = mount_container(container);
    if image_mount_point.is_empty() {
        return Ok(());
    }
    let info_file = format!("{}/{}.yaml", image_mount_point, container_basename);

    let data = fs::read_to_string(&info_file).context(format!("Failed to read {info_file}"))?;
    println!("{data}");
    umount_container(container);
    Ok(())
}

pub fn export(root: &Path, flake: &str, target: &Path) -> Result<()> {
    let config = load_from_target(Some(root), &FLAKE_DIR.join(flake)).context("failed to read flake config")?;
    if config.engine().pilot() != "podman" {
        bail!("Can only export podman flakes. This is a {} flake", config.engine().pilot())
    }
    let image = config.runtime().image_name();

    println!("Root {}  flake {}  target {}", root.to_string_lossy(), flake, target.to_string_lossy());

    let tmp = tempdir().context("Failed to create tmp dir")?;
    let tmp_target = tmp.path().join_ignore_abs(flake);
    println!("{}", tmp_target.to_string_lossy());
    let status = Command::new("podman")
        .arg("save")
        .arg("--format")
        .arg("oci-archive")
        .arg("-o")
        .arg(tmp_target)
        .arg(image)
        .status()
        .context("Failed to export oci-archive")?;
    if !status.success() {
        bail!("Failed to export oci-archive");
    }
    let status = Command::new("mv").arg(tmp.path().join(flake)).arg(target).status()?;
    if !status.success() {
        bail!("Failed to move oci-archive to target location");
    }
    Ok(())
}
