use std::fs;
use std::path::Path;
use std::os::unix::fs::symlink;
use crate::{defaults, podman, app_config};
use glob::glob;

pub fn register(container: &String, app: &String, target: Option<&String>) {
    /*!
    Register container application.

    The registration is two fold. First it will create an app symlink
    pointing to the oci-pilot launcher. Second it will create an
    app configuration file as CONTAINER_FLAKE_DIR/app.yaml containing
    the required information to launch the application inside of
    the container as follows:

    container_name: container
    program_name: target | app
    !*/
    let host_app_path = app;
    let mut target_app_path = host_app_path;
    if ! target.is_none() {
        target_app_path = target.unwrap();
    }
    for path in &[host_app_path, target_app_path] {
        if ! path.starts_with("/") {
            error!(
                "Application {:?} must be specified with an absolute path", path
            );
            return
        }
    }
    info!("Registering application: {}", host_app_path);

    // host_app_path -> pointing to oci-pilot
    match symlink(defaults::PILOT, host_app_path) {
        Ok(link) => link,
        Err(error) => {
            error!("Error while creating symlink \"{} -> {}\": {:?}",
                host_app_path, defaults::PILOT, error
            );
            return
        }
    }

    // creating default app configuration
    let app_basename = Path::new(app).file_name().unwrap().to_str().unwrap();
    let app_config_file = format!("{}/{}.yaml",
        defaults::CONTAINER_FLAKE_DIR, &app_basename
    );
    let app_config_dir = format!("{}/{}.d",
        defaults::CONTAINER_FLAKE_DIR, &app_basename
    );
    let app_config = format!(
        "container_name: {}\nprogram_name: {}\n", &container, &target_app_path
    );
    match fs::create_dir_all(&app_config_dir) {
        Ok(dir) => dir,
        Err(error) => {
            error!("Failed creating: {}: {:?}", &app_config_dir, error);
            return
        }
    };
    match fs::write(&app_config_file, app_config) {
        Ok(write) => write,
        Err(error) => {
            error!("Error creating: {}: {:?}", &app_config_file, error);
            return
        }
    }
}

pub fn remove(app: &str) {
    /*!
    Delete application link and config files
    !*/
    // remove app link
    match fs::remove_file(app) {
        Ok(remove_file) => remove_file,
        Err(error) => {
            error!("Error removing link: {}: {:?}", app, error);
        }
    }

    // remove config file and config directory
    let app_basename = Path::new(app).file_name().unwrap().to_str().unwrap();
    let app_config_dir = format!("{}/{}.d",
        defaults::CONTAINER_FLAKE_DIR, &app_basename
    );
    match fs::remove_dir_all(&&app_config_dir) {
        Ok(()) => {}
        Err(e) => { 
            error!("Error removing the config directory for the application {}: {:?}",app,e);
        }
    }
}

pub fn purge(container: &str) {
    /*!
    Iterate over all yaml config files and find those connected
    to the container. Delete all app registrations for this
    container and also delete the container from the local
    registry
    !*/
    let glob_pattern = format!("{}/*.yaml", defaults::CONTAINER_FLAKE_DIR);
    for conf_file in glob( &glob_pattern ).unwrap(){
        // load yaml config and get container name and extract app name from path
        match conf_file {
            // clean conf file and links
            Ok(path) => {
                // purge container
                podman::rm(&container.to_string());

                let pth = Path::new(&path);
                let app_basename = match  &pth.file_name().unwrap().to_str().unwrap().split(".").next() {
                    Some(v) => v,
                    None => "",
                };
                let app_conf = match app_config::AppConfig::new(&pth) {
                    Ok(r) => r,
                    Err(e) => {
                        error!("Could not load or parse the file {}: {:?}", pth.display(), e);
                        continue;
                    }
                };
                
                if container == app_conf.container_name {
                    let app = format!("{}/{}",defaults::CONTAINER_FLAKE_DIR, app_basename);
                    remove(&app);
                }
            },
            Err(e) => error!("Error while traversing configuration folder: {:?}", e),
        }
    }
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
