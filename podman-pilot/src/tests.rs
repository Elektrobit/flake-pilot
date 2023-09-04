use crate::app_path::basename;
use crate::app_path::program_abs_path;

#[test]
fn test_program_abs_path() {
    let program_path = program_abs_path();
    assert!(program_path.starts_with('/'));
}

#[test]
fn test_basename() {
    let base_name = basename(&"/some/name".to_string());
    assert_eq!("name", base_name);
}
