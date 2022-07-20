use which::which;
use std::env;
use crate::app_path::program_abs_path;
use crate::app_path::basename;
use crate::app_path::program_config_file;
use crate::container_link::container_name;

#[test]
fn test_program_abs_path() {
    let program_path = program_abs_path();
    assert!(program_path.starts_with("/"));
}

#[test]
fn test_program_config_file() {
    let config_file = program_config_file(&format!("app"));
    assert_eq!("/usr/share/flakes/app.yaml", config_file);
}

#[test]
fn test_basename() {
    let base_name = basename(&format!("/some/name"));
    assert_eq!("name", base_name);
}

#[test]
fn test_container_name() {
    let mut program_path = String::new();
    program_path.push_str(
        which("../oci-pilot_test/usr/sbin/apt-get").unwrap().to_str().unwrap()
    );
    let container = container_name(&program_path);
    assert_eq!("ubdevtools", container)
}

#[test]
#[should_panic(expected="Must be a container symlink")]
fn test_container_name_no_symlink() {
    let args: Vec<String> = env::args().collect();
    let mut program_path = String::new();
    program_path.push_str(
        which(&args[0]).unwrap().to_str().unwrap()
    );
    container_name(&program_path);
}
