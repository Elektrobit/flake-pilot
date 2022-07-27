use crate::app_path::program_abs_path;
use crate::app_path::basename;
use crate::app_path::program_config_file;

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
