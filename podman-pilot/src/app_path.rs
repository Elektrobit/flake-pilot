extern crate yaml_rust;

use std::env;
use std::path::Path;
use which::which;

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
    program_name.push_str(Path::new(program_path).file_name().unwrap().to_str().unwrap());
    program_name
}
