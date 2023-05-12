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
extern crate yaml_rust;

use std::env;
use which::which;
use std::path::Path;
use std::process::exit;
use std::fs;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

use crate::defaults;

pub fn program_abs_path() -> String {
    /*!
    Lookup absolute program path on the filesystem from
    the argv binary name of the caller
    !*/
    let args: Vec<String> = env::args().collect();
    let mut program_path = String::new();
    program_path.push_str(which(&args[0]).unwrap().to_str().unwrap());
    program_path
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

pub fn program_config_file(program_basename: &String) -> String {
    /*!
    Provide expected config file path for the given program_basename
    !*/
    let config_file = &format!(
        "{}/{}.yaml", defaults::FIRECRACKER_FLAKE_DIR, program_basename
    );
    config_file.to_string()
}

pub fn program_config_dir(program_basename: &String) -> String {
    /*!
    Provide expected config directory for the given program_basename
    !*/
    let config_dir = &format!(
        "{}/{}.d", defaults::FIRECRACKER_FLAKE_DIR, program_basename
    );
    config_dir.to_string()
}

pub fn program_config(program_basename: &String) -> Vec<Yaml> {
    /*!
    Read vm runtime configuration for given program

    FIRECRACKER_FLAKE_DIR/
       ├── program_name.d
       │   └── other.yaml
       └── program_name.yaml

    Config files below program_name.d are read in alpha sort order
    and attached to the master program_name.yaml file. The result
    is send to the Yaml parser
    !*/
    let config_file = program_config_file(program_basename);
    let mut yaml_content: String = fs::read_to_string(
        &config_file
    ).unwrap_or_else(|why| {
        error!("Failed to read: {}: {:?}", config_file, why.kind());
        exit(1)
    });
    let custom_config_dir = program_config_dir(&program_basename);
    if Path::new(&custom_config_dir).exists() {
        // put dir entries to vector to allow for sorting
        let mut custom_configs: Vec<_> = fs::read_dir(&custom_config_dir)
            .unwrap().map(|r| r.unwrap()).collect();
        custom_configs.sort_by_key(|entry| entry.path());
        for filename in custom_configs {
            let config_file = format!("{}", filename.path().display());
            let add_yaml_content: String = fs::read_to_string(
                &config_file
            ).unwrap_or_else(|why| {
                error!("Failed to read: {}: {:?}", config_file, why.kind());
                exit(1)
            });
            yaml_content.push_str(&add_yaml_content);
        }
    }
    YamlLoader::load_from_str(&yaml_content).unwrap()
}
