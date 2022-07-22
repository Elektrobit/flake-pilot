use std::fs;
use std::path::Path;
use std::os::unix::fs::symlink;
use crate::defaults;

pub fn register(container: &String, app: &String, target: Option<&String>) {
    /*!
    Register container application as a symlink structure.
    If target is not specified (app path the same on container and host) as:

    $ app -> CONTAINER_FLAKE_DIR/container@app -> PILOT

    If target is specified (app path different between host and container) as:

    $ app -> CONTAINER_FLAKE_DIR/container@target -> PILOT

    app and optional target has to be specified as absolute paths.
    Because "/" is an invalid character in symlink names it will
    be replaced by a "|" and handled correctly in PILOT when
    reading the symlink.
    !*/
    let mut host_app_path = app;
    if ! target.is_none() {
        host_app_path = target.unwrap();
    }
    for path in &[app, host_app_path] {
        if ! path.starts_with("/") {
            error!(
                "Application {:?} must be specified with an absolute path", path
            );
            return
        }
    }
    // turn container app path in symlink friendly format, replace '/' with '|'
    let container_app_path: String = app.chars().map(|c| match c {
        '/' => '|',
        _ => c
    }).collect();

    let flake_container = format!(
        "{}/{}@{}", defaults::CONTAINER_FLAKE_DIR, container, container_app_path
    );
    info!("Registering application: {}", host_app_path);

    // host_app_path -> pointing to container@container_app_path
    symlink(&flake_container, host_app_path).unwrap_or_else(|why| {
        error!("Error while creating symlink \"{} -> {}\": {:?}",
            host_app_path, flake_container, why.kind()
        );
        return
    });

    // container@container_app_path -> pointing to pilot
    if ! Path::new(&flake_container).exists() {
        symlink(defaults::PILOT, &flake_container).unwrap_or_else(|why| {
            error!("Error while creating symlink \"{} -> {}\": {:?}",
                flake_container, defaults::PILOT, why.kind()
            );
        })
    }
}

pub fn remove(app: &str) {
    // TODO: implement removal of symlink setup for registered app
    println!("remove: {:?}", app);
}

pub fn purge(container: &str) {
    // TODO: implement removal of all app registered against
    // the given container and also purge the container from
    // the local registry
    println!("purge: {:?}", container);
}

pub fn init() -> bool {
    /*!
    Create required directory structure.

    Symlink references to containers will be stored in /usr/share/flakes.
    The init method makes sure to create this directory unless it
    already exists. The way oci-pilot manages container applications
    is called a 'flake' :)
    !*/
    let mut status = true;
    fs::create_dir_all(defaults::CONTAINER_FLAKE_DIR).unwrap_or_else(|why| {
        error!(
            "Failed creating {}: {:?}",
            defaults::CONTAINER_FLAKE_DIR, why.kind()
        );
        status = false
    });
    status
}
