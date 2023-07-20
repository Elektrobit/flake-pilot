//
// Copyright (c) 2022 Elektrobit Automotive GmbH
//
// This file is part of flake-pilot
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
use crate::config::{config, RuntimeSection};
use crate::defaults::debug;
use spinoff::{spinners, Color, Spinner};
use std::env;
use std::fs;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::{Read, Write};
use std::path::Path;
use std::process::exit;
use std::process::{Command, Stdio};
use tempfile::tempfile;

use crate::defaults;

pub fn create(program_name: &String) -> Vec<String> {
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

    container:
      name: name
      target_app_path: path/to/program/in/container
      host_app_path: path/to/program/on/host

      # Optional base container to use with a delta 'container: name'
      # If specified the given 'container: name' is expected to be
      # an overlay for the specified base_container. podman-pilot
      # combines the 'container: name' with the base_container into
      # one overlay and starts the result as a container instance
      #
      # Default: not_specified
      base_container: name

      # Optional additional container layers on top of the
      # specified base container
      layers:
        - name_A
        - name_B

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
        # If the container is still running, the app will be
        # executed inside of this container instance.
        #
        # Default: false
        resume: true|false

        # Attach to the container if still running, rather than
        # executing the app again. Only makes sense for interactive
        # sessions like a shell running as app in the container.
        #
        # Default: false
        attach: true|false

        podman:
          - --storage-opt size=10G
          - --rm
          - -ti

      Calling this method returns a vector including the
      container ID and and the name of the container ID
      file.

    include:
      tar:
        - tar-archive-file-name-to-include
    !*/
    let args: Vec<String> = env::args().collect();
    let mut result: Vec<String> = Vec::new();
    let mut layers: Vec<String> = Vec::new();

    // setup container ID file name
    let mut container_cid_file = format!("{}/{}", defaults::CONTAINER_CID_DIR, program_name);
    for arg in &args[1..] {
        if arg.starts_with('@') {
            // The special @NAME argument is not passed to the
            // actual call and can be used to run different container
            // instances for the same application
            container_cid_file = format!("{}{}", container_cid_file, arg);
        }
    }
    container_cid_file = format!("{}.cid", container_cid_file);

    // check for includes
    let has_includes = config().tar.is_empty();

    // setup podman container to use
    let container_name = config().container.name;

    // setup base container if specified
    let container_base_name = config().container.base_container.unwrap_or_default();
    let delta_container = if config().container.base_container.is_some() {
        // get base container name
        // get additional container layers
        layers.extend(
            config()
                .container
                .layers
                .iter()
                .inspect(|layer| debug(&format!("Adding layer: [{}]", layer)))
                .map(|layer| (*layer).to_owned()),
        );
        true
    } else {
        false
    };

    // setup app command path name to call
    let target_app_path = get_target_app_path(program_name);

    // get runtime section

    let RuntimeSection {
        runas,
        resume,
        attach,
        ..
    } = config().runtime();

    let runas = runas.to_owned();

    let mut app = Command::new("sudo");
    if !runas.is_empty() {
        app.arg("--user").arg(&runas);
    }
    app.arg("podman")
        .arg("create")
        .arg("--cidfile")
        .arg(&container_cid_file);

    // Make sure CID dir exists
    init_cid_dir();

    // Check early return condition in resume mode
    if Path::new(&container_cid_file).exists()
        && gc_cid_file(&container_cid_file, &runas)
        && (resume || attach)
    {
        // resume or attach mode is active and container exists
        // report ID value and its ID file name
        match fs::read_to_string(&container_cid_file) {
            Ok(cid) => {
                result.push(cid);
            }
            Err(error) => {
                // cid file exists but could not be read
                panic!("Error reading CID: {:?}", error);
            }
        }
        result.push(container_cid_file);
        return result;
    }

    // Garbage collect occasionally
    gc(&runas);

    // Sanity check
    if Path::new(&container_cid_file).exists() {
        // we are about to create a container for which a
        // cid file already exists. podman create will fail with
        // an error but will also create the container which is
        // unwanted. Thus we check this condition here
        error!("Container id in use by another instance, consider @NAME argument");
        exit(1)
    }

    // create the container with configured runtime arguments
    let has_runtime_arguments = config().container.runtime.is_some();

    if let Some(runtime_section) = config().container.runtime.as_ref() {
        app.args(
            runtime_section
                .podman
                .iter()
                .flat_map(|cmd| cmd.splitn(2, ' ')),
        );
    }

    // setup default runtime arguments if not configured
    if !has_runtime_arguments {
        if resume {
            app.arg("-ti");
        } else {
            app.arg("--rm").arg("-ti");
        }
    }

    // setup container name to use
    if delta_container {
        app.arg(container_base_name);
    } else {
        app.arg(container_name);
    }

    // setup entry point
    if resume {
        // create the container with a sleep entry point
        // to keep it in running state
        app.arg("sleep");
    } else if target_app_path != "/" {
        app.arg(target_app_path);
    }

    // setup program arguments
    if resume {
        // sleep "forever" ... I will be dead by the time this sleep ends ;)
        // keeps the container in running state to accept podman exec for
        // running the app multiple times with different arguments
        app.arg("4294967295d");
    } else {
        for arg in &args[1..] {
            if !arg.starts_with('@') {
                app.arg(arg);
            }
        }
    }

    // create container
    debug(&format!("{:?}", app.get_args()));
    let spinner = Spinner::new(spinners::Line, "Launching flake...", Color::Yellow);
    match app.output() {
        Ok(output) => {
            if output.status.success() {
                let cid = String::from_utf8_lossy(&output.stdout)
                    .strip_suffix('\n')
                    .unwrap()
                    .to_string();
                result.push(cid);
                result.push(container_cid_file);

                if delta_container || has_includes {
                    debug("Mounting instance for provisioning workload");
                    let mut provision_ok = true;
                    let instance_mount_point = mount_container(&result[0], &runas, false);

                    if delta_container {
                        // Create tmpfile to hold accumulated removed data
                        let removed_files: File;
                        match tempfile() {
                            Ok(file) => removed_files = file,
                            Err(error) => {
                                spinner.fail("Flake launch has failed");
                                panic!("Failed to create tempfile: {}", error)
                            }
                        }
                        debug("Provisioning delta container...");
                        update_removed_files(&instance_mount_point, &removed_files);
                        debug(&format!(
                            "Adding main app [{}] to layer list",
                            container_name
                        ));
                        layers.push(container_name.to_string());
                        for layer in layers {
                            debug(&format!("Syncing delta dependencies [{}]...", layer));
                            let app_mount_point = mount_container(&layer, &runas, true);
                            update_removed_files(&app_mount_point, &removed_files);
                            provision_ok =
                                sync_delta(&app_mount_point, &instance_mount_point, &runas);
                            umount_container(&layer, &runas, true);
                            if !provision_ok {
                                break;
                            }
                        }
                        if provision_ok {
                            debug("Syncing host dependencies...");
                            provision_ok = sync_host(&instance_mount_point, &removed_files, &runas)
                        }
                        umount_container(&result[0], &runas, false);
                    }

                    if has_includes && provision_ok {
                        debug("Syncing includes...");
                        provision_ok = sync_includes(&instance_mount_point, &runas)
                    }

                    if !provision_ok {
                        spinner.fail("Flake launch has failed");
                        panic!("Failed to provision container")
                    }
                }
                spinner.success("Launching flake");
                return result;
            }
            spinner.fail("Flake launch has failed");
            panic!(
                "Failed to create container: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(error) => {
            spinner.fail("Flake launch has failed");
            panic!("Failed to execute podman: {:?}", error)
        }
    }
}

pub fn start(program_name: &str, cid: &str) {
    /*!
    Start container with the given container ID

    podman-pilot exits with the return code from podman after this function
    !*/
    let RuntimeSection {
        runas,
        resume,
        attach,
        ..
    } = config().runtime();
    let runas = runas.to_owned();

    let is_running = container_running(cid, &runas);

    let status_code = if is_running {
        if attach {
            // 1. Attach to running container
            call_instance("attach", cid, program_name, &runas)
        } else {
            // 2. Execute app in running container
            call_instance("exec", cid, program_name, &runas)
        }
    } else if resume {
        // 3. Startup resume type container and execute app
        let status_code = call_instance("start", cid, program_name, &runas);
        if status_code == 0 {
            call_instance("exec", cid, program_name, &runas)
        } else {
            status_code
        }
    } else {
        // 4. Startup container
        call_instance("start", cid, program_name, &runas)
    };

    exit(status_code)
}

pub fn get_target_app_path(program_name: &str) -> String {
    /*!
    setup application command path name

    This is either the program name specified at registration
    time or the configured target application from the flake
    configuration file
    !*/

    config()
        .container
        .target_app_path
        .unwrap_or(program_name)
        .to_owned()
}

pub fn call_instance(action: &str, cid: &str, program_name: &str, user: &str) -> i32 {
    /*!
    Call container ID based podman commands
    !*/
    let args: Vec<String> = env::args().collect();

    let RuntimeSection { resume, .. } = config().runtime();

    let mut call = Command::new("sudo");
    if action == "create" || action == "rm" {
        call.stderr(Stdio::null());
        call.stdout(Stdio::null());
    }
    if !user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("podman").arg(action);
    if action == "exec" {
        call.arg("--interactive");
        call.arg("--tty");
    }
    if action == "start" && !resume {
        call.arg("--attach");
    } else if action == "start" {
        // start detached, we are not interested in the
        // start output in this case
        call.stdout(Stdio::null());
    }
    call.arg(cid);
    if action == "exec" {
        call.arg(get_target_app_path(program_name));
        for arg in &args[1..] {
            if !arg.starts_with('@') {
                call.arg(arg);
            }
        }
    }
    let mut status_code = 255;
    debug(&format!("{:?}", call.get_args()));
    match call.status() {
        Ok(status) => {
            status_code = status.code().unwrap();
        }
        Err(error) => {
            error!("Failed to execute podman {}: {:?}", action, error)
        }
    }
    status_code
}

pub fn mount_container(container_name: &str, user: &String, as_image: bool) -> String {
    /*!
    Mount container and return mount point
    !*/
    let mut call = Command::new("sudo");
    if !user.is_empty() {
        call.arg("--user").arg(user);
    }
    if as_image {
        if !container_image_exists(container_name, user) {
            pull(container_name, user);
        }
        call.arg("podman")
            .arg("image")
            .arg("mount")
            .arg(container_name);
    } else {
        call.arg("podman").arg("mount").arg(container_name);
    }
    debug(&format!("{:?}", call.get_args()));
    match call.output() {
        Ok(output) => {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout)
                    .strip_suffix('\n')
                    .unwrap()
                    .to_string();
            }
            panic!(
                "Failed to mount container image: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Err(error) => {
            panic!("Failed to execute podman: {:?}", error)
        }
    }
}

pub fn umount_container(mount_point: &str, user: &String, as_image: bool) -> i32 {
    /*!
    Umount container image
    !*/
    let mut call = Command::new("sudo");
    call.stderr(Stdio::null());
    call.stdout(Stdio::null());
    if !user.is_empty() {
        call.arg("--user").arg(user);
    }
    if as_image {
        call.arg("podman")
            .arg("image")
            .arg("umount")
            .arg(mount_point);
    } else {
        call.arg("podman").arg("umount").arg(mount_point);
    }
    let mut status_code = 255;
    debug(&format!("{:?}", call.get_args()));
    match call.status() {
        Ok(status) => {
            status_code = status.code().unwrap();
        }
        Err(error) => {
            error!("Failed to execute podman image umount: {:?}", error)
        }
    }
    status_code
}

pub fn sync_includes(target: &String, user: &String) -> bool {
    /*!
    Sync custom include data to target path
    !*/

    let mut status_code = 0;
    for tar in config().tar.iter() {
        debug(&format!("Adding tar include: [{}]", tar));
        let mut call = Command::new("sudo");
        if !user.is_empty() {
            call.arg("--user").arg(user);
        }
        call.arg("tar").arg("-C").arg(target).arg("-xf").arg(tar);
        debug(&format!("{:?}", call.get_args()));
        match call.output() {
            Ok(output) => {
                debug(&String::from_utf8_lossy(&output.stdout));
                debug(&String::from_utf8_lossy(&output.stderr));
                status_code = output.status.code().unwrap();
            }
            Err(error) => {
                panic!("Failed to execute tar: {:?}", error)
            }
        }
    }

    if status_code == 0 {
        return true;
    }
    false
}

pub fn sync_delta(source: &String, target: &String, user: &String) -> bool {
    /*!
    Sync data from source path to target path
    !*/
    let mut call = Command::new("sudo");
    if !user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("rsync")
        .arg("-av")
        .arg(format!("{}/", &source))
        .arg(format!("{}/", &target));
    let status_code;
    debug(&format!("{:?}", call.get_args()));
    match call.output() {
        Ok(output) => {
            debug(&String::from_utf8_lossy(&output.stdout));
            status_code = output.status.code().unwrap();
        }
        Err(error) => {
            panic!("Failed to execute rsync: {:?}", error)
        }
    }
    if status_code == 0 {
        return true;
    }
    false
}

pub fn sync_host(target: &String, mut removed_files: &File, user: &String) -> bool {
    /*!
    Sync files/dirs specified in target/defaults::HOST_DEPENDENCIES
    from the running host to the target path
    !*/
    let mut removed_files_contents = String::new();
    let host_deps = format!("{}/{}", &target, defaults::HOST_DEPENDENCIES);
    removed_files.seek(SeekFrom::Start(0)).unwrap();
    match removed_files.read_to_string(&mut removed_files_contents) {
        Ok(_) => {
            if removed_files_contents.is_empty() {
                debug("There are no host dependencies to resolve");
                return true;
            }
            match File::create(&host_deps) {
                Ok(mut removed) => match removed.write_all(removed_files_contents.as_bytes()) {
                    Ok(_) => {}
                    Err(error) => {
                        panic!("Write failed {}: {:?}", host_deps, error);
                    }
                },
                Err(error) => {
                    panic!("Error creating {}: {:?}", host_deps, error);
                }
            }
        }
        Err(error) => {
            panic!("Error reading from file descriptor: {:?}", error);
        }
    }
    let mut call = Command::new("sudo");
    if !user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("rsync")
        .arg("-av")
        .arg("--ignore-missing-args")
        .arg("--files-from")
        .arg(&host_deps)
        .arg("/")
        .arg(format!("{}/", &target));
    let status_code;
    debug(&format!("{:?}", call.get_args()));
    match call.output() {
        Ok(output) => {
            debug(&String::from_utf8_lossy(&output.stdout));
            status_code = output.status.code().unwrap();
        }
        Err(error) => {
            panic!("Failed to execute rsync: {:?}", error)
        }
    }
    if status_code == 0 {
        return true;
    }
    false
}

pub fn init_cid_dir() {
    if !Path::new(defaults::CONTAINER_CID_DIR).is_dir() {
        if !chmod(defaults::CONTAINER_DIR, "755", "root") {
            panic!(
                "Failed to set permissions 755 on {}",
                defaults::CONTAINER_DIR
            );
        }
        if !mkdir(defaults::CONTAINER_CID_DIR, "777", "root") {
            panic!("Failed to create CID dir: {}", defaults::CONTAINER_CID_DIR);
        }
    }
}

pub fn container_running(cid: &str, user: &String) -> bool {
    /*!
    Check if container with specified cid is running
    !*/
    let mut running_status = false;
    let mut running = Command::new("sudo");
    if !user.is_empty() {
        running.arg("--user").arg(user);
    }
    running
        .arg("podman")
        .arg("ps")
        .arg("--format")
        .arg("{{.ID}}");
    debug(&format!("{:?}", running.get_args()));
    match running.output() {
        Ok(output) => {
            let mut running_cids = String::new();
            running_cids.push_str(&String::from_utf8_lossy(&output.stdout));
            for running_cid in running_cids.lines() {
                if cid.starts_with(running_cid) {
                    running_status = true;
                    break;
                }
            }
        }
        Err(error) => {
            panic!("Failed to execute podman ps: {:?}", error)
        }
    }
    running_status
}

pub fn container_image_exists(name: &str, user: &str) -> bool {
    /*!
    Check if container image is present in local registry
    !*/
    let mut exists_status = false;
    let mut exists = Command::new("sudo");
    if !user.is_empty() {
        exists.arg("--user").arg(user);
    }
    exists.arg("podman").arg("image").arg("exists").arg(name);
    debug(&format!("{:?}", exists.get_args()));
    match exists.status() {
        Ok(status) => {
            if status.code().unwrap() == 0 {
                exists_status = true
            }
        }
        Err(error) => {
            panic!("Failed to execute podman image exists: {:?}", error)
        }
    }
    exists_status
}

pub fn pull(uri: &str, user: &str) {
    /*!
    Call podman pull and prune with the provided uri
    !*/
    let mut pull = Command::new("sudo");
    if !user.is_empty() {
        pull.arg("--user").arg(user);
    }
    pull.arg("podman").arg("pull").arg(uri);
    debug(&format!("{:?}", pull.get_args()));
    match pull.output() {
        Ok(output) => {
            if !output.status.success() {
                panic!(
                    "Failed to fetch container: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else {
                let mut prune = Command::new("sudo");
                if !user.is_empty() {
                    prune.arg("--user").arg(user);
                }
                prune.arg("podman").arg("image").arg("prune").arg("--force");
                match prune.status() {
                    Ok(status) => debug(&format!("{:?}", status)),
                    Err(error) => debug(&format!("{:?}", error)),
                }
            }
        }
        Err(error) => {
            panic!("Failed to call podman pull: {}", error)
        }
    }
}

pub fn update_removed_files(target: &String, mut accumulated_file: &File) {
    /*!
    Take the contents of the given removed_file and append it
    to the accumulated_file
    !*/
    let host_deps = format!("{}/{}", &target, defaults::HOST_DEPENDENCIES);
    debug(&format!("Looking up host deps from {}", host_deps));
    if Path::new(&host_deps).exists() {
        match fs::read_to_string(&host_deps) {
            Ok(data) => {
                debug("Adding host deps...");
                debug(&String::from_utf8_lossy(data.as_bytes()));
                match accumulated_file.write_all(data.as_bytes()) {
                    Ok(_) => {}
                    Err(error) => {
                        panic!("Writing to descriptor failed: {:?}", error);
                    }
                }
            }
            Err(error) => {
                // host_deps file exists but could not be read
                panic!("Error reading {}: {:?}", host_deps, error);
            }
        }
    }
}

pub fn gc_cid_file(container_cid_file: &String, user: &String) -> bool {
    /*!
    Check if container exists according to the specified
    container_cid_file. Garbage cleanup the container_cid_file
    if no longer present. Return a true value if the container
    exists, in any other case return false.
    !*/
    let mut cid_status = false;
    match fs::read_to_string(container_cid_file) {
        Ok(cid) => {
            let mut exists = Command::new("sudo");
            if !user.is_empty() {
                exists.arg("--user").arg(user);
            }
            exists
                .arg("podman")
                .arg("container")
                .arg("exists")
                .arg(&cid);
            match exists.status() {
                Ok(status) => {
                    if status.code().unwrap() != 0 {
                        match fs::remove_file(container_cid_file) {
                            Ok(_) => {}
                            Err(error) => {
                                error!("Failed to remove CID: {:?}", error)
                            }
                        }
                    } else {
                        cid_status = true
                    }
                }
                Err(error) => {
                    error!("Failed to execute podman container exists: {:?}", error)
                }
            }
        }
        Err(error) => {
            error!("Error reading CID: {:?}", error);
        }
    }
    cid_status
}

pub fn chmod(filename: &str, mode: &str, user: &str) -> bool {
    /*!
    Chmod filename via sudo
    !*/
    let mut call = Command::new("sudo");
    if !user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("chmod").arg(mode).arg(filename);
    match call.status() {
        Ok(_) => {}
        Err(error) => {
            error!("Failed to chmod: {}: {:?}", filename, error);
            return false;
        }
    }
    true
}

pub fn mkdir(dirname: &str, mode: &str, user: &str) -> bool {
    /*!
    Make directory via sudo
    !*/
    let mut call = Command::new("sudo");
    if !user.is_empty() {
        call.arg("--user").arg(user);
    }
    call.arg("mkdir").arg("-p").arg("-m").arg(mode).arg(dirname);
    match call.status() {
        Ok(_) => {}
        Err(error) => {
            error!("Failed to mkdir: {}: {:?}", dirname, error);
            return false;
        }
    }
    true
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
        cid_file_count += 1;
    }
    if cid_file_count <= defaults::GC_THRESHOLD {
        return;
    }
    for container_cid_file in cid_file_names {
        gc_cid_file(&container_cid_file, user);
    }
}
