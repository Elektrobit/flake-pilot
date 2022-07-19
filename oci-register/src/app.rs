use std::fs;
use crate::defaults;

pub fn register(container: &String, app: &String, target: Option<&String>) {
    // TODO: implement symlink setup for app registration
    println!("register: {:?} {:?} {:?}", container, app, target);
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
