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
use std::ffi::OsStr;
use std::process::Command;
use tempfile::tempdir;
use std::path::Path;
use crate::defaults;
use crate::{app, app_config};
use std::fs;

use crate::fetch::{fetch_file, send_request};

pub fn init_toplevel_image_dir(image_dir: &str) -> bool {
    /*!
    Create toplevel firecracker image storage directory
    !*/
    let mut ok = false;
    if ! Path::new(&image_dir).exists() {
        match fs::create_dir_all(&image_dir) {
            Ok(_) => { ok = true },
            Err(error) => {
                error!("Error creating directory {}: {}", image_dir, error);
            }
        }
    } else {
        ok = true
    }
    ok
}

pub async fn pull_kis_image(name: &String, uri: &String, force: bool) -> i32 {
    /*!
    Fetch the data provided in uri and treat it as a KIWI
    built KIS image type. This means the file behind uri
    is expected to be a tarball containing the KIS
    components; rootfs-image, kernel and optional initrd
    !*/
    let mut result = 255;

    info!("Fetching KIS image...");

    if ! init_toplevel_image_dir(defaults::FIRECRACKER_IMAGES_DIR) {
        return result
    }

    let image_dir = format!("{}/{}", defaults::FIRECRACKER_IMAGES_DIR, name);
    if force && Path::new(&image_dir).exists() {
        match fs::remove_dir_all(&image_dir) {
            Ok(_) => { },
            Err(error) => {
                error!("Error removing directory {}: {}", image_dir, error);
                return result
            }
        }
    }
    if Path::new(&image_dir).exists() {
        error!("Image directory '{}' already exists", image_dir);
        return result
    }
    match tempdir() {
        Ok(tmp_dir) => {
            let work_dir = tmp_dir.path().join("work")
                .into_os_string().into_string().unwrap();
            let kis_tar = tmp_dir.path().join("kis_archive")
                .into_os_string().into_string().unwrap();

            // Download...
            match fs::create_dir_all(&work_dir) {
                Ok(_) => {
                    match send_request(&uri).await {
                        Ok(response) => {
                            result = response.status().as_u16().into();
                            match fetch_file(response, &kis_tar).await {
                                Ok(_) => { },
                                Err(error) => {
                                    error!("Download failed with: {}", error);
                                    return result
                                }
                            }
                        },
                        Err(error) => {
                            error!(
                                "Request to '{}' failed with: {}", uri, error
                            );
                            return result
                        }
                    }
                },
                Err(error) => {
                    error!(
                        "Error creating work directory {}: {}",
                        work_dir, error
                    );
                    return result
                }
            }

            // Unpack and Rename...
            info!("Unpacking...");
            let mut tar = Command::new("tar");
            tar.arg("-C").arg(&work_dir)
               .arg("-xf").arg(&kis_tar);
            match tar.status() {
                Ok(status) => {
                    result = status.code().unwrap();
                },
                Err(error) => {
                    error!("Failed to execute tar: {:?}", error);
                    return result
                }
            }
            let mut kis_ok = 4;
            for path in fs::read_dir(&work_dir).unwrap() {
                let path = path.unwrap().path();
                let extension = path.extension().unwrap();
                if extension == OsStr::new("append") {
                    fs::remove_file(&path).unwrap();
                    kis_ok -= 1;
                } else if extension == OsStr::new("md5") {
                    fs::remove_file(&path).unwrap();
                    kis_ok -= 1;
                } else if extension == OsStr::new("initrd") {
                    fs::rename(&path, format!("{}/{}",
                        work_dir, defaults::FIRECRACKER_INITRD_NAME
                    )).unwrap();
                    // optional
                } else if extension == OsStr::new("kernel") {
                    fs::rename(&path, format!("{}/{}",
                        work_dir, defaults::FIRECRACKER_KERNEL_NAME
                    )).unwrap();
                    kis_ok -= 1;
                } else {
                    fs::rename(&path, format!("{}/{}",
                        work_dir, defaults::FIRECRACKER_ROOTFS_NAME
                    )).unwrap();
                    kis_ok -= 1;
                }
            }
            if kis_ok != 0 {
                error!("Not a KIWI kis type image");
                return result
            }

            // Move to final firecracker image store
            match fs::rename(&work_dir, &image_dir) {
                Ok(_) => { },
                Err(error) => {
                    error!(
                        "Failed to rename {} -> {}: {:?}",
                        work_dir, image_dir, error
                    );
                    return result
                }
            }
        },
        Err(error) => {
            error!("Failed to create tempdir: {}", error);
            return result
        }
    }
    result
}

pub fn purge_vm(vm: &str) {
    /*!
    Iterate over all yaml config files and find those connected
    to the VM. Delete all app registrations for this
    VM and also delete the VM from the local registry
    !*/
    for app_name in app::app_names() {
        let config_file = format!(
            "{}/{}.yaml", defaults::FLAKE_DIR, app_name
        );
        match app_config::AppConfig::init_from_file(Path::new(&config_file)) {
            Ok(mut app_conf) => {
                if vm == app_conf.vm.as_mut().unwrap().name {
                    app::remove(
                        &app_conf.vm.as_mut().unwrap().host_app_path,
                        defaults::FIRECRACKER_PILOT, false
                    );
                }
            },
            Err(error) => {
                error!(
                    "Ignoring error on load or parse flake config {}: {:?}",
                    config_file, error
                );
            }
        };
    }
    let image_dir = format!("{}/{}", defaults::FIRECRACKER_IMAGES_DIR, vm);
    match fs::remove_dir_all(&image_dir) {
        Ok(_) => { },
        Err(error) => {
            error!("Error removing directory {}: {}", image_dir, error);
        }
    }
}
