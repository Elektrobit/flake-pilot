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
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use tempfile::tempdir;
use std::path::Path;
use std::borrow::Cow;
use std::fs;

use crate::defaults;
use crate::{app, app_config};

use crate::fetch::{fetch_file, send_request};

pub fn init_toplevel_image_dir(registry_dir: &str) -> bool {
    /*!
    Create firecracker registry directory layout
    !*/
    let mut ok = true;
    let mut real_registry_dir = String::new();
    match fs::read_link(registry_dir) {
        Ok(target) => {
            real_registry_dir.push_str(
                &target.into_os_string().into_string().unwrap()
            );
        },
        Err(_) => {
            real_registry_dir.push_str(registry_dir);
        }
    }
    let mut subdirs: Vec<String> = Vec::new();
    subdirs.push(format!("{}/images", real_registry_dir));
    subdirs.push(format!("{}/storage", real_registry_dir));
    for subdir in subdirs {
        if Path::new(&subdir).exists() {
            continue
        }
        match fs::create_dir_all(&subdir) {
            Ok(_) => {
                match fs::metadata(&subdir) {
                    Ok(attr) => {
                        let mut permissions = attr.permissions();
                        permissions.set_mode(0o777);
                        match fs::set_permissions(&subdir, permissions) {
                            Ok(_) => { },
                            Err(error) => {
                                error!(
                                    "Failed to set 0o777 bits: {} {}",
                                    &subdir, error
                                );
                                ok = false
                            }
                        }
                    },
                    Err(error) => {
                        error!(
                            "Failed to fetch attributes: {} {}", &subdir, error
                        );
                        ok = false
                    }
                }
            },
            Err(error) => {
                error!("Error creating directory {}: {}", &subdir, error);
                ok = false
            }
        }
    }
    ok
}

pub async fn pull_component_image(
    name: &String, rootfs_uri: Option<&String>, kernel_uri: Option<&String>,
    initrd_uri: Option<&String>, force: bool
) -> i32 {
    /*!
    Fetch components image consisting out of rootfs, kernel and
    optional initrd.
    !*/
    let mut result = 255;
    let image_dir = format!("{}/{}", defaults::FIRECRACKER_IMAGES_DIR, name);
    struct Component<'a> {
        uri: String,
        file: Cow<'a, str>
    }
    info!("Fetching Component image...");
    if ! pull_new(name, force) {
        return result
    }
    match tempdir() {
        Ok(tmp_dir) => {
            let mut download_files: Vec<Component> = Vec::new();
            let rootfs_file = tmp_dir.path().join("rootfs")
                .into_os_string().into_string().unwrap();
            let kernel_file = tmp_dir.path().join("kernel")
                .into_os_string().into_string().unwrap();
            let initrd_file = tmp_dir.path().join("initrd")
                .into_os_string().into_string().unwrap();
            download_files.push(
                Component {
                    uri: rootfs_uri.unwrap().to_string(),
                    file: Cow::Borrowed(&rootfs_file),
                }
            );
            download_files.push(
                Component {
                    uri: kernel_uri.unwrap().to_string(),
                    file: Cow::Borrowed(&kernel_file),
                }
            );
            if initrd_uri.is_some() {
                download_files.push(
                    Component {
                        uri: initrd_uri.unwrap().to_string(),
                        file: Cow::Borrowed(&initrd_file),
                    }
                );
            }
            // Download...
            for component in download_files {
                match send_request(&component.uri).await {
                    Ok(response) => {
                        result = response.status().as_u16().into();
                        match fetch_file(
                            response, &component.file.into_owned()).await
                        {
                            Ok(_) => { },
                            Err(error) => {
                                error!(
                                    "Download failed with: {}", error
                                );
                                return result
                            }
                        }
                    },
                    Err(error) => {
                        error!(
                            "Request to '{}' failed with: {}",
                            component.uri, error
                        );
                        return result
                    }
                }
            }
            // Check for sci and add it to rootfs image if not present
            let tmp_dir_path = tmp_dir.path().display().to_string();
            if mount_fs_image(&rootfs_file, &tmp_dir_path, "root") {
                let sci_in_image = format!(
                    "{}/{}", tmp_dir_path, "/usr/sbin/sci"
                );
                let overlay_root_in_image = format!(
                    "{}/{}", tmp_dir_path, "/overlayroot"
                );
                if ! Path::new(&sci_in_image).exists() {
                    info!("Copying sci to rootfs...");
                    if ! copy(
                        defaults::FIRECRACKER_SCI, &sci_in_image, "root"
                    ) {
                        umount(&tmp_dir_path, "root");
                        return result
                    }
                }
                if ! Path::new(&overlay_root_in_image).exists() && ! mkdir(&overlay_root_in_image, "root") {
                    umount(&tmp_dir_path, "root");
                    return result
                }
                umount(&tmp_dir_path, "root");
            }

            // Move to final firecracker image store
            if ! mv(&tmp_dir_path, &image_dir, "root") {
                return result
            }
        },
        Err(error) => {
            error!("Failed to create tempdir: {}", error);
            return result
        }
    }
    result
}

pub async fn pull_kis_image(
    name: &String, uri: Option<&String>, force: bool
) -> i32 {
    /*!
    Fetch the data provided in uri and treat it as a KIWI
    built KIS image type. This means the file behind uri
    is expected to be a tarball containing the KIS
    components; rootfs-image, kernel and optional initrd
    !*/
    let mut result = 255;
    let image_dir = format!("{}/{}", defaults::FIRECRACKER_IMAGES_DIR, name);

    info!("Fetching KIS image...");

    if ! pull_new(name, force) {
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
                    match send_request(uri.unwrap()).await {
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
                                "Request to '{}' failed with: {}",
                                uri.unwrap(), error
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
            if ! mv(&work_dir, &image_dir, "root") {
                return result
            }
        },
        Err(error) => {
            error!("Failed to create tempdir: {}", error);
            return result
        }
    }
    result
}

pub fn mkdir(dirname: &String, user: &str) -> bool {
    /*!
    Make directory via sudo
    !*/
    let mut call = Command::new("sudo");
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("mkdir").arg("-p").arg(dirname);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to mkdir: {}: {:?}", dirname, error);
            return false
        }
    }
    true
}

pub fn mv(source: &str, target: &String, user: &str) -> bool {
    /*!
    Move file via sudo
    !*/
    let mut call = Command::new("sudo");
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("mv").arg(source).arg(target);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to mv: {} -> {}: {:?}", source, target, error);
            return false
        }
    }
    true
}

pub fn copy(source: &str, target: &String, user: &str) -> bool {
    /*!
    Copy file via sudo
    !*/
    let mut call = Command::new("sudo");
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("cp").arg(source).arg(target);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to cp: {} -> {}: {:?}", source, target, error);
            return false
        }
    }
    true
}

pub fn mount_fs_image(
    fs_name: &str, mount_point: &String, user: &str
) -> bool {
    /*!
    Mount filesystem image
    !*/
    let mut call = Command::new("sudo");
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("mount").arg(fs_name).arg(mount_point);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to execute mount: {:?}", error);
            return false
        }
    }
    true
}

pub fn umount(mount_point: &str, user: &str) -> bool {
    /*!
    Umount given mount_point
    !*/
    let mut call = Command::new("sudo");
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("umount").arg(mount_point);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to execute mount: {:?}", error);
            return false
        }
    }
    true
}


pub fn pull_new(name: &String, force: bool) -> bool {
    /*!
    Initialize new pull
    !*/
    if ! init_toplevel_image_dir(defaults::FIRECRACKER_REGISTRY_DIR) {
        return false
    }
    let image_dir = format!("{}/{}", defaults::FIRECRACKER_IMAGES_DIR, name);
    if force && Path::new(&image_dir).exists() {
        match fs::remove_dir_all(&image_dir) {
            Ok(_) => { },
            Err(error) => {
                error!("Error removing directory {}: {}", image_dir, error);
                return false
            }
        }
    }
    if Path::new(&image_dir).exists() {
        error!("Image directory '{}' already exists", image_dir);
        return false
    }
    true
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
                if app_conf.vm.is_some() &&
                    vm == app_conf.vm.as_mut().unwrap().name
                {
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
