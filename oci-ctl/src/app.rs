//
// Copyright (c) 2022 Elektrobit Automotive GmbH
//
// This file is part of oci-pilot
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
use std::fs;
use std::path::Path;
use std::os::unix::fs::symlink;
use crate::{defaults, podman, app_config};
use glob::glob;

pub fn register(
    container: &String, app: &String,
    target: Option<&String>, base: Option<&String>,
    layers: Option<Vec<String>>, resume: Option<&bool>,
    attach: Option<&bool>
) {
    /*!
    Register container application.

    The registration is two fold. First it will create an app symlink
    pointing to the oci-pilot launcher. Second it will create an
    app configuration file as CONTAINER_FLAKE_DIR/app.yaml containing
    the required information to launch the application inside of
    the container.
    !*/
    let host_app_path = app;
    let mut target_app_path = host_app_path;
    if ! target.is_none() {
        target_app_path = target.unwrap();
    }
    for path in &[host_app_path, target_app_path] {
        if ! path.starts_with("/") {
            error!(
                "Application {:?} must be specified with an absolute path", path
            );
            return
        }
    }
    if base.is_none() && ! layers.is_none() {
        error!("Container layer(s) specified without a base container");
        return
    }
    info!("Registering application: {}", host_app_path);

    // host_app_path -> pointing to oci-pilot
    match symlink(defaults::PILOT, host_app_path) {
        Ok(link) => link,
        Err(error) => {
            error!("Error while creating symlink \"{} -> {}\": {:?}",
                host_app_path, defaults::PILOT, error
            );
            return
        }
    }

    // creating default app configuration
    let app_basename = Path::new(app).file_name().unwrap().to_str().unwrap();
    let app_config_file = format!("{}/{}.yaml",
        defaults::CONTAINER_FLAKE_DIR, &app_basename
    );
    let app_config_dir = format!("{}/{}.d",
        defaults::CONTAINER_FLAKE_DIR, &app_basename
    );
    match fs::create_dir_all(&app_config_dir) {
        Ok(dir) => dir,
        Err(error) => {
            error!("Failed creating: {}: {:?}", &app_config_dir, error);
            return
        }
    };
    match app_config::AppConfig::save(
        Path::new(&app_config_file),
        &container, &target_app_path, &host_app_path,
        base, layers, resume, attach
    ) {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to create AppConfig {}: {:?}",
                app_config_file, error
            );
        }
    }
}

pub fn remove(app: &str) {
    /*!
    Delete application link and config files
    !*/
    if ! app.starts_with("/") {
        error!(
            "Application {:?} must be specified with an absolute path", app
        );
        return
    }
    info!("Removing application: {}", app);
    // remove pilot link if valid
    match fs::read_link(app) {
        Ok(link_name) => {
            if link_name.into_os_string() == defaults::PILOT {
                match fs::remove_file(app) {
                    Ok(_) => {},
                    Err(error) => {
                        error!("Error removing pilot link: {}: {:?}", app, error);
                        return
                    }
                }
            } else {
                error!("Symlink not pointing to oci-pilot: {}", app);
                return
            }
        },
        Err(error) => {
            error!("Failed to read as symlink: {}: {:?}", app, error);
            return
        }
    }
    // remove config file and config directory
    let app_basename = basename(&format!("{}",app));
    let config_file = format!(
        "{}/{}.yaml", defaults::CONTAINER_FLAKE_DIR, &app_basename
    );
    let app_config_dir = format!(
        "{}/{}.d", defaults::CONTAINER_FLAKE_DIR, &app_basename
    );
    if Path::new(&config_file).exists() {
        match fs::remove_file(&config_file) {
            Ok(_) => {},
            Err(error) => {
                error!(
                    "Error removing config file: {}: {:?}",
                    config_file, error
                )
            }
        }
    }
    if Path::new(&app_config_dir).exists() {
        match fs::remove_dir_all(&app_config_dir) {
            Ok(_) => {},
            Err(error) => {
                error!(
                    "Error removing config directory: {}: {:?}",
                    app_config_dir , error
                )
            }
        }
    }
}

pub fn basename(program_path: &String) -> String {
    /*!
    Get basename from given program path
    !*/
    let mut program_name = String::new();
    program_name.push_str(
        Path::new(program_path).file_name().unwrap().to_str().unwrap()
    );
    program_name
}

pub fn app_names() -> Vec<String> {
    /*!
    Read all flake config files
    !*/
    let mut flakes: Vec<String> = Vec::new();
    let glob_pattern = format!("{}/*.yaml", defaults::CONTAINER_FLAKE_DIR);
    for config_file in glob(&glob_pattern).unwrap() {
        match config_file {
            Ok(filepath) => {
                let base_config_file = basename(
                    &filepath.into_os_string().into_string().unwrap()
                );
                match base_config_file.split(".").next() {
                    Some(value) => {
                        let mut app_name = String::new();
                        app_name.push_str(value);
                        flakes.push(app_name);
                    },
                    None => error!(
                        "Ignoring invalid config_file format: {}",
                        base_config_file
                    )
                }
            },
            Err(error) => error!(
                "Error while traversing flakes folder: {:?}", error
            )
        }
    }
    flakes
}

pub fn purge(container: &str) {
    /*!
    Iterate over all yaml config files and find those connected
    to the container. Delete all app registrations for this
    container and also delete the container from the local
    registry
    !*/
    for app_name in app_names() {
        let config_file = format!(
            "{}/{}.yaml", defaults::CONTAINER_FLAKE_DIR, app_name
        );
        match app_config::AppConfig::init_from_file(Path::new(&config_file)) {
            Ok(app_conf) => {
                if container == app_conf.container {
                    remove(&app_conf.host_app_path);
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
    podman::rm(&container.to_string());
}

pub fn init() -> bool {
    /*!
    Create required directory structure.

    Symlink references to containers will be stored in /usr/share/flakes.
    The init method makes sure to create this directory unless it
    already exists. The way oci-pilot manages container applications
    is called a 'flake' :)
    !*/
    let mut status = true;
    fs::create_dir_all(defaults::CONTAINER_FLAKE_DIR).unwrap_or_else(|why| {
        error!(
            "Failed creating {}: {:?}",
            defaults::CONTAINER_FLAKE_DIR, why.kind()
        );
        status = false
    });
    status
}
