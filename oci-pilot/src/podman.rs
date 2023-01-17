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
use std::path::Path;
use std::process::{Command, Stdio};
use std::os::unix::fs::PermissionsExt;
use std::process::exit;
use std::env;
use std::fs;
use crate::app_path::program_config;
use crate::app_path::program_config_file;

use crate::defaults;

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

    container: name
    target_app_path: path/to/program/in/container
    host_app_path: path/to/program/on/host

    runtime:
      # Run the container engine as a user other than the
      # default target user root. The user may be either
      # a user name or a numeric user-ID (UID) prefixed
      # with the ‘#’ character (e.g. #0 for UID 0). The call
      # of the container engine is performed by sudo.
      # The behavior of sudo can be controlled via the
      # file /etc/sudoers
      runas: root

      # Try to resume container from previous execution.
      # If the container is still running, the call will attach to it
      # If the container is not running, the call will restart the
      # container and attach to it.
      #
      # NOTE: If processing the call inside of the container finishes
      # faster than attaching to it, the call will run a new container
      # which is attached immediately. If this is unwanted set:
      # respawn: false
      #
      # Default: false
      resume: true|false

      # Run a new container if attaching to resumed container failed
      # This setting is only effective if 'resume: true' is set
      #
      # Default: true
      respawn: true|false

      podman:
        - --storage-opt size=10G
        - --rm:
        - -ti:

    Calling this method will exit the calling program with the
    exit code from podman or 255 in case no exit code can be
    obtained
    !*/
    let args: Vec<String> = env::args().collect();

    let runtime_config = program_config(&program_name);
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

    // setup podman app to call
    let mut target_app_path = program_name.as_str();
    if ! runtime_config[0]["target_app_path"].as_str().is_none() {
        target_app_path = runtime_config[0]["target_app_path"].as_str().unwrap();
    }

    // get runtime section
    let runtime_section = &runtime_config[0]["runtime"];

    // setup container operation mode
    let mut resume: bool = false;
    let mut respawn: bool = true;
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

    let mut app = Command::new("sudo");
    if ! runas.is_empty() {
        app.arg("--user").arg(&runas);
    }
    app.arg("podman");

    if resume {
        // Make sure CID dir exists
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

        // Garbage collect occasionally
        gc(&runas);

        if ! Path::new(&container_cid_file).exists() {
            // resume mode is active and container doesn't exist
            // run the container and init a new ID file
            app.arg("run");
            app.arg("--cidfile");
            app.arg(container_cid_file);
        } else {
            // resume mode is active and container exists
            // either attach or restart to it
            match fs::read_to_string(&container_cid_file) {
                Ok(cid) => {
                    // debug!("{:?}", cid);
                    // 1. try to attach to the container
                    if resume_instance("attach", &cid, &runas) == 0 {
                        exit(0)
                    }
                    // 2. try to restart/attach to the container
                    if resume_instance("restart", &cid, &runas) == 0 {
                        if resume_instance("attach", &cid, &runas) == 0 {
                            exit(0)
                        } else if ! respawn {
                            // attach failed, mostly because the process
                            // started already finished. By default we
                            // run a new container that is attached immediately
                            // but no respawn indicates this is unwanted.
                            // So we are leaving...
                            exit(0)
                        }
                    }
                    // 3. resume failed, delete instance and CID and re-init
                    resume_instance("rm", &cid, &runas);
                    match fs::remove_file(&container_cid_file) {
                        Ok(_) => { },
                        Err(error) => {
                            error!("Failed to remove CID: {:?}", error)
                        }
                    }
                    app.arg("run");
                    app.arg("--cidfile");
                    app.arg(container_cid_file);
                },
                Err(error) => {
                    // cid file exists but could not be read
                    // fallback to start a new container without CID
                    error!("Error reading CID: {:?}", error);
                    app.arg("run");
                }
            }
        }
    } else {
        app.arg("run");
    }

    // run the container and application
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
    if ! has_runtime_arguments {
        app.arg("--rm").arg("-ti");
    }

    app.arg(container_name);

    if target_app_path != "/" {
        app.arg(target_app_path);
    }

    // setup program arguments
    for arg in &args[1..] {
        if ! arg.starts_with("@") {
            app.arg(arg);
        }
    }

    // debug!("{:?}", app.get_args());
    match app.status() {
        Ok(status) => {
            match status.code() {
                Some(code) => exit(code),
                None => panic!("Process terminated by signal")
            }
        },
        Err(error) => error!("Failed to execute podman: {:?}", error)
    }
    exit(255)
}

pub fn resume_instance(action: &str, cid: &String, user: &String) -> i32 {
    /*!
    Call container ID based podman commands
    !*/
    let mut resume = Command::new("sudo");
    resume.stderr(Stdio::null());
    if action == "restart" || action == "rm" {
        resume.stdout(Stdio::null());
    }
    if ! user.is_empty() {
        resume.arg("--user").arg(user);
    }
    resume.arg("podman").arg(action).arg(&cid);
    let mut status_code = 255;
    match resume.status() {
        Ok(status) => {
            status_code = status.code().unwrap();
        },
        Err(error) => {
            error!("Failed to execute podman {}: {:?}", action, error)
        }
    }
    status_code
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
