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
use crate::{app_config, defaults, firecracker};
use glob::glob;
use log::{error, info};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

pub fn register(app: Option<&String>, target: Option<&String>, engine: &str) -> bool {
    /*!

    Create an app symlink pointing to the engine launcher.
    !*/
    if app.is_none() {
        error!("No application specified");
        return false;
    }
    let host_app_path = app.unwrap();
    let target_app_path = target.unwrap_or(host_app_path);
    for path in &[host_app_path, target_app_path] {
        if !path.starts_with('/') {
            error!(
                "Application {:?} must be specified with an absolute path",
                path
            );
            return false;
        }
    }
    info!("Registering application: {}", host_app_path);

    // host_app_path -> pointing to engine
    let host_app_dir = Path::new(host_app_path).parent().unwrap().to_str().unwrap();
    match fs::create_dir_all(host_app_dir) {
        Ok(dir) => dir,
        Err(error) => {
            error!("Failed creating: {}: {:?}", &host_app_dir, error);
            return false;
        }
    };
    match symlink(engine, host_app_path) {
        Ok(link) => link,
        Err(error) => {
            error!(
                "Error while creating symlink \"{} -> {}\": {:?}",
                host_app_path, &engine, error
            );
            return false;
        }
    }

    // creating default app configuration
    let app_basename = Path::new(app.unwrap())
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let app_config_dir = format!("{}/{}.d", defaults::FLAKE_DIR, &app_basename);
    match fs::create_dir_all(&app_config_dir) {
        Ok(dir) => dir,
        Err(error) => {
            error!("Failed creating: {}: {:?}", &app_config_dir, error);
            return false;
        }
    }
    true
}

#[allow(clippy::too_many_arguments)]
pub fn create_vm_config(
    vm: &String,
    app: Option<&String>,
    target: Option<&String>,
    run_as: Option<&String>,
    overlay_size: Option<&String>,
    no_net: bool,
    resume: bool,
    includes_tar: Option<Vec<String>>,
) -> bool {
    /*!
    Create app configuration for the firecracker engine.

    Create an app configuration file as FLAKE_DIR/app.yaml
    containing the required information to launch the
    application inside of the firecracker engine.
    !*/
    
    let host_app_path = app.unwrap();
    let target_app_path = target.unwrap_or(host_app_path);
    let app_basename = Path::new(host_app_path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap();
    let app_config_file = format!("{}/{}.yaml", defaults::FLAKE_DIR, &app_basename);
    match app_config::AppConfig::save_vm(
        Path::new(&app_config_file),
        vm,
        target_app_path,
        host_app_path,
        run_as,
        overlay_size,
        no_net,
        resume,
        includes_tar,
    ) {
        Ok(_) => true,
        Err(error) => {
            error!(
                "Failed to create AppConfig {}: {:?}",
                app_config_file, error
            );
            false
        }
    }
}

pub fn remove(app: &str, engine: &str, silent: bool) {
    /*!
    Delete application link and config files
    !*/
    if !app.starts_with('/') {
        if !silent {
            error!(
                "Application {:?} must be specified with an absolute path",
                app
            );
        }
        return;
    }
    if !silent {
        info!("Removing application: {}", app);
    }
    // remove pilot link if valid
    match fs::read_link(app) {
        Ok(link_name) => {
            if link_name.into_os_string() == engine {
                match fs::remove_file(app) {
                    Ok(_) => {}
                    Err(error) => {
                        if !silent {
                            error!("Error removing pilot link: {}: {:?}", app, error);
                        }
                        return;
                    }
                }
            } else {
                if !silent {
                    error!("Symlink not pointing to {}: {}", engine, app);
                }
                return;
            }
        }
        Err(error) => {
            if !silent {
                error!("Failed to read as symlink: {}: {:?}", app, error);
            }
            return;
        }
    }
    // remove config file and config directory
    let app_basename = basename(&app.to_string());
    let config_file = format!("{}/{}.yaml", defaults::FLAKE_DIR, &app_basename);
    let app_config_dir = format!("{}/{}.d", defaults::FLAKE_DIR, &app_basename);
    if Path::new(&config_file).exists() {
        match fs::remove_file(&config_file) {
            Ok(_) => {}
            Err(error) => {
                if !silent {
                    error!("Error removing config file: {}: {:?}", config_file, error)
                }
            }
        }
    }
    if Path::new(&app_config_dir).exists() {
        match fs::remove_dir_all(&app_config_dir) {
            Ok(_) => {}
            Err(error) => {
                if !silent {
                    error!(
                        "Error removing config directory: {}: {:?}",
                        app_config_dir, error
                    )
                }
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
        Path::new(program_path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap(),
    );
    program_name
}

pub fn app_names() -> Vec<String> {
    /*!
    Read all flake config files
    !*/
    let mut flakes: Vec<String> = Vec::new();
    let glob_pattern = format!("{}/*.yaml", defaults::FLAKE_DIR);
    for config_file in glob(&glob_pattern).unwrap() {
        match config_file {
            Ok(filepath) => {
                let base_config_file = basename(&filepath.into_os_string().into_string().unwrap());
                match base_config_file.split('.').next() {
                    Some(value) => {
                        let mut app_name = String::new();
                        app_name.push_str(value);
                        flakes.push(app_name);
                    }
                    None => error!("Ignoring invalid config_file format: {}", base_config_file),
                }
            }
            Err(error) => error!("Error while traversing flakes folder: {:?}", error),
        }
    }
    flakes
}

pub fn purge(app: &str, engine: &str) {
    /*!
    Iterate over all yaml config files and delete all app
    registrations and its connected resources for the specified app
    !*/
    if engine == defaults::FIRECRACKER_PILOT {
        firecracker::purge_vm(app)
    }
}

pub fn init(app: Option<&String>) -> bool {
    /*!
    Create required directory structure.

    Symlink references to apps will be stored in defaults::FLAKE_DIR
    The init method makes sure to create this directory unless it
    already exists.
    !*/
    let mut status = true;
    if app.is_some() && Path::new(&app.unwrap()).exists() {
        error!("App path {} already exists", app.unwrap());
        return false;
    }
    let mut flake_dir = String::new();
    match fs::read_link(defaults::FLAKE_DIR) {
        Ok(target) => {
            flake_dir.push_str(&target.into_os_string().into_string().unwrap());
        }
        Err(_) => {
            flake_dir.push_str(defaults::FLAKE_DIR);
        }
    }
    fs::create_dir_all(flake_dir).unwrap_or_else(|why| {
        error!("Failed creating {}: {:?}", defaults::FLAKE_DIR, why.kind());
        status = false
    });
    status
}
