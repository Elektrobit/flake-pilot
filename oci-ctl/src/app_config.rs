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
extern crate yaml_rust;

use yaml_rust::YamlLoader;

// constants related to field names in configuration 
const CONTAINER:&str = "container";
const TARGET_APP_PATH:&str = "target_app_path";
const HOST_APP_PATH:&str = "host_app_path";

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

/*
AppConfig represents application yaml configuration
*/
pub struct AppConfig {
    pub container: String,
    pub target_app_path: String,
    pub host_app_path: String,
    pub config_file: String
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
        // TODO: handling yaml string can be done better here...
        let mut app_config = format!(
            "container: {}\ntarget_app_path: {}\nhost_app_path: {}\n",
            &container,
            &target_app_path,
            &host_app_path
        );
        if ! base.is_none() {
            app_config = format!(
                "{}base_container: {}\n", app_config, base.unwrap()
            );
        }
        if ! layers.is_none() {
            app_config = format!("{}layer:\n", app_config);
            for layer in layers.unwrap() {
                app_config = format!("{}  - {}\n", app_config, layer)
            }
            app_config = format!("{}\n", app_config);
        }
        fs::write(&config_file, app_config)?;
        Ok(())
    }

    pub fn init_from_file(config_file: &Path) -> Result<AppConfig, GenericError> {
        /*!
        new creates the new AppConfig class by reading and
        deserializing the data from a given yaml configuration
        !*/
        let mut rs = AppConfig{
            container: "".to_string(),
            target_app_path: "".to_string(),
            host_app_path: "".to_string(),
            config_file: config_file.display().to_string()
        };

        if Path::new(&config_file).exists() {
            let source = fs::read_to_string(&config_file)?;
            let doc = YamlLoader::load_from_str(&source)?;

            rs.container = match doc[0][CONTAINER].as_str() {
                Some(v) => v.to_string(),
                None => "".to_string()
            };

            rs.target_app_path = match doc[0][TARGET_APP_PATH].as_str() {
                Some(v) => v.to_string(),
                None => "".to_string()
            };

            rs.host_app_path = match doc[0][HOST_APP_PATH].as_str() {
                Some(v) => v.to_string(),
                None => "".to_string()
            };
        }
        Ok(rs)
    }
}
