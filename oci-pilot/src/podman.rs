use std::process::Command;
use std::process::exit;
use std::path::Path;
use std::env;
use crate::app_path::program_config;
use crate::app_path::program_config_file;
use crate::app_path::basename;
use crate::defaults;

pub fn run(program_path: &String, container_name: &String) {
    /*!
    Call podman run and execute program_path inside of container_name
    All commandline options will be passed to the program
    called in the container. Options to control how podman starts
    the container can be provided as CONTAINER_FLAKE_DIR/program_name.yaml
    like the following example shows:

    runtime:
      --storage-opt: size=10G
      --rm:

    If no runtime configuration exists the following defaults apply

    - Container resources will be automatically deleted after the call.
    - Interactive sessions will be allowed

    Calling this method will exit the calling program with the
    exit code from podman or 255 in case no exit code can be
    obtained
    !*/
    let program_name = basename(&program_path);
    let args: Vec<String> = env::args().collect();

    let mut app = Command::new(defaults::PODMAN);

    // setup podman runtime arguments
    app.arg("run");
    if Path::new(&program_config_file(&program_name)).exists() {
        let runtime_config = program_config(&program_name);
        let runtime_section = &runtime_config[0]["runtime"];
        for (opt, opt_value) in runtime_section.as_hash().unwrap() {
            app.arg(opt.as_str().unwrap());
            if ! opt_value.as_str().is_none() {
                app.arg(opt_value.as_str().unwrap());
            }
        }
    } else {
        app.arg("--rm").arg("-ti");
    }
    app.arg(container_name).arg(program_path);

    // setup program arguments
    for arg in &args[1..] {
        app.arg(arg);
    }

    let status = app.status()
        .expect(&format!("Failed to execute {}", defaults::PODMAN));

    match status.code() {
        Some(code) => exit(code),
        None => println!("Process terminated by signal")
    }
    exit(255);
}
