use std::process::Command;
use std::process::exit;
use std::path::Path;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let program_path = &args[0];
    let program_link_path = fs::read_link(program_path)
        .expect(&format!("Not a symlink: {}", program_path));

    // set programname from binary path
    let program_name = Path::new(program_path).file_name().unwrap().to_str().unwrap();

    // set containername from symlink parent
    let mut container_name = String::new();
    container_name.push_str(
        Path::new(&program_link_path).parent().unwrap().to_str().unwrap()
    );

    // allow to override containername via @CONTAINERNAME
    for arg in &args[1..] {
        if arg.starts_with("@") {
            container_name = String::from(&arg.replace("@", ""));
        }
    }

    let mut app = Command::new("podman");

    app.arg("run")
        .arg("--rm")
        .arg("-ti")
        .arg(container_name)
        .arg(program_name);

    for arg in &args[1..] {
        if ! arg.starts_with("@") {
            app.arg(arg);
        }
    }

    let status = app.status()
        .expect(&format!("Failed to execute: {}", program_path));

    match status.code() {
        Some(code) => exit(code),
        None => println!("Process terminated by signal")
    }
}
