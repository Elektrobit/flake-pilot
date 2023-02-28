//
// Copyright (c) 2022 Elektrobit Automotive GmbH
//
// This file is part of oci-pilot
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.
//
use yaml_rust::Yaml;
use std::path::Path;
use std::process::{Command, Stdio};
use std::os::unix::fs::PermissionsExt;
use std::process::exit;
use std::env;
use std::fs;
use crate::app_path::program_config_file;

use crate::defaults;

pub fn create(program_name: &String, runtime_config: &Vec<Yaml>) -> Vec<String> {
    /*!
    Create container for later execution of program_name.
    The container name and all other settings to run the program
    inside of the container are taken from the config file(s)

    CONTAINER_FLAKE_DIR/
       ├── program_name.d
       │   └── other.yaml
       └── program_name.yaml

    All commandline options will be passed to the program_name
    called in the container. An example program config file
    looks like the following:

    container: name
    target_app_path: path/to/program/in/container
    host_app_path: path/to/program/on/host

    # Optional base container to use with a delta 'container: name'
    # If specified the given 'container: name' is expected to be
    # an overlay for the specified base_container. oci-pilot
    # combines the 'container: name' with the base_container into
    # one overlay and starts the result as a container instance
    #
    # Default: not_specified
    base_container: name

    runtime:
      # Run the container engine as a user other than the
      # default target user root. The user may be either
      # a user name or a numeric user-ID (UID) prefixed
      # with the ‘#’ character (e.g. #0 for UID 0). The call
      # of the container engine is performed by sudo.
      # The behavior of sudo can be controlled via the
      # file /etc/sudoers
      runas: root

      # Resume the container from previous execution.
      # If the container is still running, the call will attach to it
      # If attaching is not possible, the container gets started again
      # and immediately attached.
      #
      # Default: false
      resume: true|false

      # Create and start a new container if attaching or startup of
      # resumed container failed. This setting is only effective
      # if 'resume: true' is set.
      #
      # Default: true
      respawn: true|false

      podman:
        - --storage-opt size=10G
        - --rm
        - -ti

    Calling this method returns a vector including the
    container ID and and the name of the container ID
    file. It depends on the flake configuration if the
    container ID file is being created or not.
    !*/
    let args: Vec<String> = env::args().collect();
    let mut result: Vec<String> = Vec::new();

    let mut container_cid_file = format!(
        "{}/{}", defaults::CONTAINER_CID_DIR, program_name
    );
    for arg in &args[1..] {
        // build container ID file specific to the caller arguments
        container_cid_file = format!("{}{}", container_cid_file, arg);
    }
    container_cid_file = format!("{}.cid", container_cid_file);

    // setup podman container to use
    if runtime_config[0]["container"].as_str().is_none() {
        error!(
            "No 'container' attribute specified in {}",
            program_config_file(&program_name)
        );
        exit(1)
    }
    let container_name = runtime_config[0]["container"].as_str().unwrap();

    // setup base container if specified
    let container_base_name;
    let delta_container;
    if ! runtime_config[0]["base_container"].as_str().is_none() {
        container_base_name = runtime_config[0]["base_container"]
            .as_str().unwrap();
        delta_container = true;
    } else {
        container_base_name = "";
        delta_container = false;
    }

    // setup podman app to call
    let mut target_app_path = program_name.as_str();
    if ! runtime_config[0]["target_app_path"].as_str().is_none() {
        target_app_path = runtime_config[0]["target_app_path"]
            .as_str().unwrap();
    }

    // get runtime section
    let runtime_section = &runtime_config[0]["runtime"];

    // setup container operation mode
    let mut resume: bool = false;
    let mut runas = String::new();

    if ! runtime_section.as_hash().is_none() {
        if ! &runtime_section["resume"].as_bool().is_none() {
            resume = runtime_section["resume"].as_bool().unwrap();
        }
        if ! &runtime_section["runas"].as_str().is_none() {
            runas.push_str(&runtime_section["runas"].as_str().unwrap());
        }
    }

    let mut app = Command::new("sudo");
    if ! runas.is_empty() {
        app.arg("--user").arg(&runas);
    }
    app.arg("podman");

    if resume {
        // Make sure CID dir exists
        init_cid_dir();

        // Garbage collect occasionally
        gc(&runas);

        if ! Path::new(&container_cid_file).exists() {
            // resume mode is active and container doesn't exist
            // create the container and init a new ID file
            app.arg("create");
            app.arg("--cidfile");
            app.arg(&container_cid_file);
        } else {
            // resume mode is active and container exists
            // report ID=exists and its ID file name
            result.push(format!("exists"));
            result.push(container_cid_file);
            return result;
        }
    } else {
        app.arg("create");
    }

    // create the container with configured runtime arguments
    let mut has_runtime_arguments: bool = false;
    if ! runtime_section.as_hash().is_none() {
        let podman_section = &runtime_section["podman"];
        if ! podman_section.as_vec().is_none() {
            has_runtime_arguments = true;
            for opt in podman_section.as_vec().unwrap() {
                let mut split_opt = opt.as_str().unwrap().splitn(2, ' ');
                let opt_name = split_opt.next();
                let opt_value = split_opt.next();
                app.arg(opt_name.unwrap());
                if ! opt_value.is_none() {
                    app.arg(opt_value.unwrap());
                }
            }
        }
    }

    // setup default runtime arguments if not configured
    if ! has_runtime_arguments {
        app.arg("--rm").arg("-ti");
    }

    // setup container name to use
    if delta_container {
        app.arg(container_base_name);
    } else {
        app.arg(container_name);
    }

    // setup entry point
    if target_app_path != "/" {
        app.arg(target_app_path);
    }

    // setup program arguments
    for arg in &args[1..] {
        if ! arg.starts_with("@") {
            app.arg(arg);
        }
    }

    info!("{:?}", app.get_args());
    match app.output() {
        Ok(output) => {
            if output.status.success() {
                let cid = String::from_utf8_lossy(&output.stdout)
                    .strip_suffix("\n").unwrap().to_string();
                result.push(cid);
                result.push(container_cid_file);
                if delta_container {
                    let app_mount_point = mount_container(
                        &container_name, &runas, true
                    );
                    let instance_mount_point = mount_container(
                        &result[0], &runas, false
                    );
                    let sync_ok = sync_delta(
                        &app_mount_point, &instance_mount_point, &runas
                    );
                    umount_container(&container_name, &runas, true);
                    umount_container(&result[0], &runas, false);
                    if sync_ok > 0 {
                        panic!("Failed to sync delta to base")
                    }
                }
                return result;
            }
            panic!(
                "Failed to create container: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        },
        Err(error) => {
            panic!("Failed to execute podman: {:?}", error)
        }
    }
}

pub fn start(
    program_name: &String, runtime_config: &Vec<Yaml>,
    cid: &String, container_cid_file: &String
) {
    /*!
    Start container with given container ID

    Calling this method will exit the calling program with the
    exit code from podman or 255 in case no exit code can be
    obtained
    !*/
    let runtime_section = &runtime_config[0]["runtime"];

    let mut status_code;
    let error_boundary_for_podman = 125;
    let mut resume: bool = false;
    let mut respawn: bool = true;
    let mut attach: bool = false;
    let mut runas = String::new();

    if ! runtime_section.as_hash().is_none() {
        if ! &runtime_section["resume"].as_bool().is_none() {
            resume = runtime_section["resume"].as_bool().unwrap();
        }
        if ! &runtime_section["respawn"].as_bool().is_none() {
            respawn = runtime_section["respawn"].as_bool().unwrap();
        }
        if ! &runtime_section["runas"].as_str().is_none() {
            runas.push_str(&runtime_section["runas"].as_str().unwrap());
        }
    }

    let container_id;
    if resume && Path::new(&container_cid_file).exists() && cid == "exists" {
        // resume mode is active and container exists, read ID file
        match fs::read_to_string(&container_cid_file) {
            Ok(cid) => {
                container_id = format!("{}", cid);
                attach = true;
            },
            Err(error) => {
                // cid file exists but could not be read
                panic!("Error reading CID: {:?}", error);
            }
        }
    } else {
        container_id = format!("{}", cid);
    }

    // 1. try to attach if existing
    if attach {
        status_code = call_instance("attach", &container_id, &runas);
        if status_code < error_boundary_for_podman {
            exit(status_code)
        }
    }

    // 2. normal startup of the container
    status_code = call_instance("start", &container_id, &runas);

    // 3. attach or start has failed, re-init and call if respawn
    if respawn && status_code >= error_boundary_for_podman {
        call_instance("rm", &cid, &runas);
        match fs::remove_file(&container_cid_file) {
            Ok(_) => { },
            Err(error) => {
                error!("Failed to remove CID: {:?}", error)
            }
        }
        let container = create(&program_name, &runtime_config);
        status_code = call_instance("start", &container[0], &runas)
    }
    exit(status_code)
}

pub fn call_instance(action: &str, cid: &String, user: &String) -> i32 {
    /*!
    Call container ID based podman commands
    !*/
    let mut call = Command::new("sudo");
    if action == "create" || action == "rm" {
        call.stderr(Stdio::null());
        call.stdout(Stdio::null());
    }
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("podman").arg(action).arg(&cid);
    if action == "start" {
        call.arg("--attach");
    }
    let mut status_code = 255;
    match call.status() {
        Ok(status) => {
            status_code = status.code().unwrap();
        },
        Err(error) => {
            error!("Failed to execute podman {}: {:?}", action, error)
        }
    }
    status_code
}

pub fn mount_container(
    container_name: &str, user: &String, as_image: bool
) -> String {
    /*!
    Mount container and return mount point
    !*/
    let mut call = Command::new("sudo");
    call.stderr(Stdio::null());
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    if as_image {
        call.arg("podman").arg("image").arg("mount").arg(&container_name);
    } else {
        call.arg("podman").arg("mount").arg(&container_name);
    }
    match call.output() {
        Ok(output) => {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout)
                    .strip_suffix("\n").unwrap().to_string()
            }
            panic!(
                "Failed to mount container image: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        },
        Err(error) => {
            panic!("Failed to execute podman: {:?}", error)
        }
    }
}

pub fn umount_container(
    mount_point: &str, user: &String, as_image: bool
) -> i32 {
    /*!
    Umount container image
    !*/
    let mut call = Command::new("sudo");
    call.stderr(Stdio::null());
    call.stdout(Stdio::null());
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    if as_image {
        call.arg("podman").arg("image").arg("umount").arg(&mount_point);
    } else {
        call.arg("podman").arg("umount").arg(&mount_point);
    }
    let mut status_code = 255;
    match call.status() {
        Ok(status) => {
            status_code = status.code().unwrap();
        },
        Err(error) => {
            error!("Failed to execute podman image umount: {:?}", error)
        }
    }
    status_code
}

pub fn sync_delta(
    source: &String, target: &String, user: &String
) -> i32 {
    /*!
    Sync data from source path to target path
    !*/
    let mut call = Command::new("sudo");
    // call.stderr(Stdio::null());
    // call.stdout(Stdio::null());
    if ! user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("rsync").arg("-a").arg(format!("{}/", &source)).arg(format!("{}/", &target));
    let mut status_code = 255;
    match call.status() {
        Ok(status) => {
            status_code = status.code().unwrap();
        },
        Err(error) => {
            error!("Failed to execute rsync: {:?}", error)
        }
    }
    status_code
}

pub fn init_cid_dir() {
    if ! Path::new(defaults::CONTAINER_CID_DIR).is_dir() {
        fs::create_dir(defaults::CONTAINER_CID_DIR).unwrap_or_else(|why| {
            panic!("Failed to create CID dir: {:?}", why.kind());
        });
        let attr = fs::metadata(
            defaults::CONTAINER_CID_DIR
        ).unwrap_or_else(|why| {
            panic!("Failed to fetch CID attributes: {:?}", why.kind());
        });
        let mut permissions = attr.permissions();
        permissions.set_mode(0o777);
        fs::set_permissions(
            defaults::CONTAINER_CID_DIR, permissions
        ).unwrap_or_else(|why| {
            panic!("Failed to set CID permissions: {:?}", why.kind());
        });
    }
}

pub fn gc(user: &String) {
    /*!
    Garbage collect CID files for which no container exists anymore
    !*/
    let mut cid_file_names: Vec<String> = Vec::new();
    let mut cid_file_count: i32 = 0;
    let paths = fs::read_dir(defaults::CONTAINER_CID_DIR).unwrap();
    for path in paths {
        cid_file_names.push(format!("{}", path.unwrap().path().display()));
        cid_file_count = cid_file_count + 1;
    }
    if cid_file_count <= defaults::GC_THRESHOLD {
        return
    }
    for container_cid_file in cid_file_names {
        match fs::read_to_string(&container_cid_file) {
            Ok(cid) => {
                let mut exists = Command::new("sudo");
                if ! user.is_empty() {
                    exists.arg("--user").arg(&user);
                }
                exists.arg("podman").arg("container").arg("exists").arg(&cid);
                match exists.status() {
                    Ok(status) => {
                        if status.code().unwrap() != 0 {
                            match fs::remove_file(&container_cid_file) {
                                Ok(_) => { },
                                Err(error) => {
                                    error!("Failed to remove CID: {:?}", error)
                                }
                            }
                        }
                    },
                    Err(error) => {
                        error!(
                            "Failed to execute podman container exists: {:?}",
                            error
                        )
                    }
                }
            },
            Err(error) => {
                error!("Error reading CID: {:?}", error);
            }
        }
    }
}
