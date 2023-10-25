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
use crate::defaults;
use anyhow::{bail, Context, Result};
use glob::glob;
use log::{error, info};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

pub fn register(host_app_path: &str, target_app_path: &str, engine: &str) -> Result<()> {
    /*!
    Register container application for specified engine.

    Create an app symlink pointing to the engine launcher.
    !*/
    for path in &[host_app_path, target_app_path] {
        if !Path::new(path).is_absolute() {
            bail!("Application {path} must be specified with an absolute path");
        }
    }
    info!("Registering application: {}", host_app_path);

    // host_app_path -> pointing to engine
    let host_app_dir = Path::new(host_app_path).parent().unwrap().to_str().unwrap();
    fs::create_dir_all(host_app_dir).context(format!("Could not create {host_app_dir}"))?;
    symlink(engine, host_app_path).context("Could not create symlink")?;

    // creating default app configuration
    let app_basename = Path::new(host_app_path).file_name().unwrap().to_str().unwrap();
    let app_config_dir = format!("{}/{}.d", defaults::FLAKE_DIR, &app_basename);
    fs::create_dir_all(app_config_dir).context("Failed to create config directory")?;
    Ok(())
}

pub fn remove(app: &Path) -> Result<()> {
    //    Delete application link and config files
    if !app.is_absolute() {
        bail!("Application must be specified with an absolute path");
    }
    info!("Removing application: {}", app.to_string_lossy());
    // remove pilot link if valid

    let pilot = fs::read_link(app).context(format!("Could not read symlink {}", app.to_string_lossy()))?;
    if pilot.file_name() == Some(OsStr::new("podman-pilot")) {
        fs::remove_file(app).context("Could not delete app")?;
    } else {
        bail!("Not a podman-pilot app")
    }

    // remove config file and config directory
    match app.file_name() {
        Some(basename) => {
            fs::remove_file(Path::new(defaults::FLAKE_DIR).join(basename).with_extension("yaml"))?;
            fs::remove_dir(Path::new(defaults::FLAKE_DIR).join(basename).with_extension("d"))?;
        }
        None => bail!("malformed app path {}", app.to_string_lossy()),
    };

    Ok(())
}

pub fn basename(program_path: &String) -> String {
    /*!
    Get basename from given program path
    !*/
    let mut program_name = String::new();
    program_name.push_str(Path::new(program_path).file_name().unwrap().to_str().unwrap());
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

pub fn init(app: &str) -> Result<()> {
    /*!
    Create required directory structure.

    Symlink references to apps will be stored in defaults::FLAKE_DIR
    The init method makes sure to create this directory unless it
    already exists.
    !*/
    if Path::new(&app).exists() {
        bail!("App path {app} already exists");
    }
    let flake_dir = fs::read_link(defaults::FLAKE_DIR).unwrap_or_else(|_| PathBuf::from(defaults::FLAKE_DIR));
    fs::create_dir_all(flake_dir).context(format!("Failed creating {}", defaults::FLAKE_DIR))
}
