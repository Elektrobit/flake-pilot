use std::process::Command;
use std::process::exit;
use std::env;

pub fn run(program_name: &String, container_name: &String) {
    /*!
    Call podman run and execute program_name inside of container_name
    All commandline options which does not start with the @ magic
    indicator for pilot options will be passed to the program_name
    called in the container.

    - Container resources will be automatically deleted after the call.
    - Interactive sessions will be allowed

    Calling this method will exit the calling program with the
    exit code from podman or 255 in case no exit code can be
    obtained
    !*/
    let args: Vec<String> = env::args().collect();
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
        .expect(&format!("Failed to execute podman"));

    match status.code() {
        Some(code) => exit(code),
        None => println!("Process terminated by signal")
    }
    exit(255);
}
