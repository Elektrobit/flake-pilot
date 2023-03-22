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
use std::path::Path;
use serde::{Serialize, Deserialize};
use serde_yaml::{self};
use crate::defaults;

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

// AppConfig represents application yaml configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub container: String,
    pub target_app_path: String,
    pub host_app_path: String,
    pub base_container: Option<String>,
    pub layers: Option<Vec<String>>,
    pub runtime: Option<AppRuntime>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppRuntime {
    pub runas: Option<String>,
    pub resume: Option<bool>,
    pub attach: Option<bool>,
    pub podman: Option<Vec<String>>
}

impl AppConfig {
    pub fn save(
        config_file: &Path, container: &String, target_app_path: &String,
        host_app_path: &String, base: Option<&String>,
        layers: Option<Vec<String>>
    ) -> Result<(), GenericError> {
        /*!
        save stores an AppConfig to the given file
        !*/
        let template = std::fs::File::open(defaults::FLAKE_TEMPLATE)
            .expect(&format!("Failed to open {}", defaults::FLAKE_TEMPLATE));
        let mut yaml_config: AppConfig = serde_yaml::from_reader(template)
            .expect("Failed to import config template");
        yaml_config.container = container.to_string();
        yaml_config.target_app_path = target_app_path.to_string();
        yaml_config.host_app_path = host_app_path.to_string();
        if ! base.is_none() {
            yaml_config.base_container = Some(base.unwrap().to_string());
        }
        if ! layers.is_none() {
            yaml_config.layers = Some(layers.as_ref().unwrap().to_vec());
        }
        let config = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&config_file)
            .expect(&format!("Failed to open {:?}", config_file));
        serde_yaml::to_writer(config, &yaml_config).unwrap();
        Ok(())
    }

    pub fn init_from_file(
        config_file: &Path
    ) -> Result<AppConfig, GenericError> {
        /*!
        new creates the new AppConfig class by reading and
        deserializing the data from a given yaml configuration
        !*/
        let config = std::fs::File::open(&config_file)
            .expect(&format!("Failed to open {:?}", config_file));
        let yaml_config: AppConfig = serde_yaml::from_reader(config)
            .expect("Failed to import config file");
        Ok(yaml_config)
    }
}
