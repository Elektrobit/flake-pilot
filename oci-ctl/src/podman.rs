use std::process::Command;

const PODMAN_PATH:&str = "/usr/bin/podman";

pub fn load(oci: &String) -> i32 {
    /*!
    Call podman load with the provided oci tar file
    !*/
    let mut status_code = 255;

    info!("Loading OCI container...");
    info!("podman load -i {}", oci);
    let status = Command::new(PODMAN_PATH)
        .arg("load")
        .arg("-i")
        .arg(oci)
        .status();

    match status {
        Ok(status) => {
            status_code = status.code().unwrap();
            if ! status.success() {
                error!("Failed, error message(s) reported");
            }
        }
        Err(status) => { error!("Process terminated by signal: {}", status) }
    }

    status_code
}

pub fn purge(container: &String){
 /*!
    Call podman image rm with force option to remove all running containers
    !*/
    info!("Removing image and all running containers...");
    info!("podman load -i {}", container);
    let status = Command::new(PODMAN_PATH)
        .arg("image")
        .arg("rm")
        .arg("-f")
        .arg(container)
        .status();

    match status {
        Ok(status) => {
            if ! status.success() {
                error!("Failed, error message(s) reported");
            }
        }
        Err(status) => { error!("Process terminated by signal: {}", status) }
    }
    
}
