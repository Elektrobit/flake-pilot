extern crate yaml_rust;

use std::env;
use which::which;
use std::path::Path;
use std::fs;
use yaml_rust::Yaml;
use yaml_rust::YamlLoader;

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
    let config_file = &format!("/etc/pilot/{}.yaml", program_basename);
    config_file.to_string()
}

pub fn program_config(program_basename: &String) -> Vec<Yaml> {
    /*!
    Read container runtime configuration for given program
    !*/
    let config_file = program_config_file(program_basename);
    let yaml_content = fs::read_to_string(&config_file)
        .expect(&format!("Failed to read program config {}", config_file));
    let yaml = YamlLoader::load_from_str(&yaml_content).unwrap();
    yaml
}
