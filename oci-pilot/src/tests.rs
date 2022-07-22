use which::which;
use std::env;
use crate::app_path::program_abs_path;
use crate::app_path::basename;
use crate::app_path::program_config_file;
use crate::container_link::read_link;

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
fn test_read_link_no_container_app_path() {
    let mut program_path = String::new();
    program_path.push_str(
        which("../oci-pilot_test/usr/sbin/apt-get").unwrap().to_str().unwrap()
    );
    let container_meta = read_link(&program_path);
    assert_eq!("ubdevtools", container_meta[0]);
}

#[test]
fn test_read_link_with_container_app_path() {
    let mut program_path = String::new();
    program_path.push_str(
        which("../oci-pilot_test/usr/sbin/lsblk").unwrap().to_str().unwrap()
    );
    let container_meta = read_link(&program_path);
    assert_eq!("ubdevtools", container_meta[0]);
    assert_eq!("/usr/sbin/lsblk", container_meta[1])
}

#[test]
#[should_panic(expected="Must be a container symlink")]
fn test_read_link_no_symlink() {
    let args: Vec<String> = env::args().collect();
    let mut program_path = String::new();
    program_path.push_str(
        which(&args[0]).unwrap().to_str().unwrap()
    );
    read_link(&program_path);
}
