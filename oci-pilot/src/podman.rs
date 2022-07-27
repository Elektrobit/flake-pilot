use std::process::Command;
use std::process::exit;
use std::path::Path;
use std::env;
use crate::app_path::program_config;
use crate::app_path::program_config_file;

pub fn run(program_name: &String) {
    /*!
    Call podman run and execute program_name inside of a container.
    The container name and all other settings to run the program
    inside of the container are taken from the config file(s)

    CONTAINER_FLAKE_DIR/
       ├── program_name.d
       │   └── other.yaml
       └── program_name.yaml

    All commandline options will be passed to the program_name
    called in the container. An example program config file
    looks like the following:

    container_name: name

    runtime:
      --storage-opt: size=10G
      --rm:
      -ti:

    Calling this method will exit the calling program with the
    exit code from podman or 255 in case no exit code can be
    obtained
    !*/
    let args: Vec<String> = env::args().collect();

    let runtime_config = program_config(&program_name);

    let mut app = Command::new("podman");

    // setup podman container to use
    if runtime_config[0]["container_name"].as_str().is_none() {
        error!(
            "No 'container_name' specified in {}",
            program_config_file(&program_name)
        );
        exit(1)
    }
    let container_name = runtime_config[0]["container_name"].as_str().unwrap();

    // setup podman runtime arguments
    app.arg("run");
    if Path::new(&program_config_file(&program_name)).exists() {
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
    app.arg(container_name).arg(program_name);

    // setup program arguments
    for arg in &args[1..] {
        app.arg(arg);
    }

    match app.status() {
        Ok(status) => {
            match status.code() {
                Some(code) => exit(code),
                None => error!("Process terminated by signal")
            }
        },
        Err(error) => error!("Failed to execute podman: {:?}", error)
    };
    exit(255);
}
