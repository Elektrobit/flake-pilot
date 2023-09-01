use flakes::user::User;
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
use crate::error::{CommandError, CommandExtTrait, FlakeError};
use nix::unistd;
use spinoff::{spinners, Color, Spinner};
use std::fs;
use std::fs::File;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, os::unix::prelude::PermissionsExt};
use tempfile::tempfile;

use crate::defaults;

/// Create container for later execution of program_name.
/// The container name and all other settings to run the program
/// inside of the container are taken from the config file(s)
///
/// CONTAINER_FLAKE_DIR/
///    ├── program_name.d
///    │   └── other.yaml
///    └── program_name.yaml
///
/// All commandline options will be passed to the program_name
/// called in the container. An example program config file
/// looks like the following:
///
/// container:
///   name: name
///   target_app_path: path/to/program/in/container
///   host_app_path: path/to/program/on/host
///
///   # Optional base container to use with a delta 'container: name'
///   # If specified the given 'container: name' is expected to be
///   # an overlay for the specified base_container. podman-pilot
///   # combines the 'container: name' with the base_container into
///   # one overlay and starts the result as a container instance
///   #
///   # Default: not_specified
///   base_container: name
///
///   # Optional additional container layers on top of the
///   # specified base container
///   layers:
///     - name_A
///     - name_B
///
///   runtime:
///     # Run the container engine as a user other than the
///     # default target user root. The user may be either
///     # a user name or a numeric user-ID (UID) prefixed
///     # with the ‘#’ character (e.g. #0 for UID 0). The call
///     # of the container engine is performed by sudo.
///     # The behavior of sudo can be controlled via the
///     # file /etc/sudoers
///     runas: root
///
///     # Resume the container from previous execution.
///     # If the container is still running, the app will be
///     # executed inside of this container instance.
///     #
///     # Default: false
///     resume: true|false
///
///     # Attach to the container if still running, rather than
///     # executing the app again. Only makes sense for interactive
///     # sessions like a shell running as app in the container.
///     #
///     # Default: false
///     attach: true|false
///
///     podman:
///       - --storage-opt size=10G
///       - --rm
///       - -ti
///
///   Calling this method returns a vector including the
///   container ID and and the name of the container ID
///   file.
///
/// include:
///   tar:
///     - tar-archive-file-name-to-include
pub fn create(program_name: &String) -> Result<(String, String), FlakeError> {
    let args: Vec<String> = env::args().collect();
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

    let container_section = &config().container;

    // check for includes
    let tar_includes = &config().tars();
    let has_includes = !tar_includes.is_empty();

    // setup podman container to use
    let container_name = container_section.name;

    // setup base container if specified
    let delta_container = container_section.base_container.is_some();
    let container_base_name = container_section.base_container.unwrap_or_default();

    if container_section.base_container.is_some() {
        // get additional container layers

        layers.extend(
            config().layers().iter().inspect(|layer| debug(&format!("Adding layer: [{layer}]"))).map(|x| (*x).to_owned()),
        );
    }

    // setup app command path name to call
    let target_app_path = get_target_app_path(program_name);

    // get runtime section
    let RuntimeSection { runas, resume, attach, .. } = config().runtime();
    let runas = runas.to_owned();

    let mut app = runas.run("podman");
    app.arg("create").arg("--cidfile").arg(&container_cid_file);

    // Make sure CID dir exists
    init_cid_dir()?;

    // Check early return condition in resume mode
    if Path::new(&container_cid_file).exists() && gc_cid_file(&container_cid_file, runas)? && (resume || attach) {
        // resume or attach mode is active and container exists
        // report ID value and its ID file name

        let cid = fs::read_to_string(&container_cid_file)?;

        return Ok((cid, container_cid_file));
    }

    // Garbage collect occasionally
    gc(runas)?;

    // Sanity check
    if Path::new(&container_cid_file).exists() {
        return Err(FlakeError::AlreadyRunning);
    }

    // create the container with configured runtime arguments
    let mut has_runtime_arguments: bool = false;
    if let Some(RuntimeSection { podman, .. }) = &config().container.runtime {
        let podman = podman.as_ref().cloned().unwrap_or_default();

        app.args(podman.iter().flat_map(|x| x.splitn(2, ' ')));
        has_runtime_arguments = !podman.is_empty();
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
    let spinner = Spinner::new_with_stream(spinners::Line, "Launching flake...", Color::Yellow, spinoff::Streams::Stderr);

    match run_podman_creation(app, delta_container, has_includes, runas, container_name, layers, &container_cid_file) {
        Ok(container) => {
            spinner.success("Launching flake");
            Ok(container)
        }
        Err(err) => {
            spinner.fail("Flake launch has failed");
            Err(err)
        }
    }
}

/// Create podman
fn run_podman_creation(
    mut app: Command, delta_container: bool, has_includes: bool, runas: User, container_name: &str, mut layers: Vec<String>,
    container_cid_file: &str,
) -> Result<(String, String), FlakeError> {
    let cid = String::from_utf8_lossy(&app.perform()?.stdout).trim_end_matches('\n').to_owned();

    if delta_container || has_includes {
        debug("Mounting instance for provisioning workload");
        let instance_mount_point = mount_container(&cid, runas, false)?;

        if delta_container {
            // Create tmpfile to hold accumulated removed data
            let removed_files = tempfile()?;

            debug("Provisioning delta container...");
            update_removed_files(&instance_mount_point, &removed_files)?;
            debug(&format!("Adding main app [{}] to layer list", container_name));
            layers.push(container_name.to_string());
            for layer in layers {
                debug(&format!("Syncing delta dependencies [{}]...", layer));
                let app_mount_point = mount_container(&layer, runas, true)?;
                update_removed_files(&app_mount_point, &removed_files)?;
                sync_delta(&app_mount_point, &instance_mount_point, runas)?;

                match umount_container(&layer, runas, true) {
                    Ok(_) => {}
                    Err(err) => {
                        warn!("{}", err);
                    }
                }
            }
            debug("Syncing host dependencies...");
            sync_host(&instance_mount_point, &removed_files, runas)?;

            match umount_container(&cid, runas, false) {
                Ok(_) => {}
                Err(err) => {
                    warn!("{}", err);
                }
            }
        }

        if has_includes {
            debug("Syncing includes...");
            sync_includes(&instance_mount_point, runas)?;
        }
    }
    Ok((cid, container_cid_file.to_owned()))
}

/// Start container with the given container ID
pub fn start(program_name: &str, cid: &str) -> Result<(), FlakeError> {
    let RuntimeSection { runas, resume, attach, .. } = config().runtime();

    if container_running(cid, runas)? {
        call_instance(if attach { "attach" } else { "exec" }, cid, program_name, runas)?;
    } else {
        call_instance("start", cid, program_name, runas)?;
        if resume {
            call_instance("exec", cid, program_name, runas)?;
        }
    }

    Ok(())
}

/// Setup application command path name
///
/// This is either the program name specified at registration
/// time or the configured target application from the flake
/// configuration file
pub fn get_target_app_path(program_name: &str) -> String {
    config().container.target_app_path.unwrap_or(program_name).to_owned()
}

/// Call container ID based podman commands
pub fn call_instance(action: &str, cid: &str, program_name: &str, user: User) -> Result<(), FlakeError> {
    let RuntimeSection { resume, .. } = config().runtime();

    let mut call = user.run("podman");
    call.arg(action);

    // Mute STDOUT/STDERR on create and removal
    if action == "create" || action == "rm" {
        call.stderr(Stdio::null());
        call.stdout(Stdio::null());
    }

    // Mute STDOUT on start when resuming
    // start detached, we are not interested in the
    // start output in this case
    if action == "start" && resume {
        call.stdout(Stdio::null());
    }

    if action == "exec" {
        call.arg("--interactive").arg("--tty");
    }

    if action == "start" && !resume {
        call.arg("--attach");
    }

    call.arg(cid);

    if action == "exec" {
        call.arg(get_target_app_path(program_name));
        for arg in &env::args().collect::<Vec<String>>()[1..] {
            if !arg.starts_with('@') {
                call.arg(arg);
            }
        }
    }

    debug(&format!("{:?}", call.get_args()));
    call.status()?;
    Ok(())
}

/// Mount container and return mount point
pub fn mount_container(container_name: &str, user: User, as_image: bool) -> Result<String, FlakeError> {
    let mut call = user.run("podman");
    if as_image {
        if !container_image_exists(container_name, user)? {
            pull(container_name, user)?;
        }
        call.arg("image");
    }
    call.arg("mount").arg(container_name);

    debug(&format!("{:?}", call.get_args()));

    Ok(String::from_utf8_lossy(&call.perform()?.stdout).trim().to_string())
}

/// Umount container image
pub fn umount_container(mount_point: &str, user: User, as_image: bool) -> Result<(), FlakeError> {
    let mut call = user.run("podman");
    call.stderr(Stdio::null());
    call.stdout(Stdio::null());

    if as_image {
        call.arg("image");
    }

    call.arg("umount").arg(mount_point);

    debug(&format!("{:?}", call.get_args()));

    call.perform()?;
    Ok(())
}

/// Sync custom include data to target path
pub fn sync_includes(target: &String, user: User) -> Result<(), FlakeError> {
    let tar_includes = &config().tars();

    for tar in tar_includes {
        debug(&format!("Adding tar include: [{}]", tar));
        let mut call = user.run("tar");
        call.arg("-C").arg(target).arg("-xf").arg(tar);
        debug(&format!("{:?}", call.get_args()));
        let output = call.perform()?;
        debug(&String::from_utf8_lossy(&output.stdout));
        debug(&String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

/// Sync data from source path to target path
pub fn sync_delta(source: &String, target: &String, user: User) -> Result<(), CommandError> {
    // XXX: This is an external dependency, not always present.
    let mut call = user.run("rsync");

    call.arg("-av").arg(format!("{}/", &source)).arg(format!("{}/", &target));
    debug(&format!("{:?}", call.get_args()));
    call.perform()?;

    Ok(())
}

/// Sync files/dirs specified in target/defaults::HOST_DEPENDENCIES
/// from the running host to the target path
pub fn sync_host(target: &String, mut removed_files: &File, user: User) -> Result<(), FlakeError> {
    let mut removed_files_contents = String::new();
    let host_deps = format!("{}/{}", &target, defaults::HOST_DEPENDENCIES);
    removed_files.seek(SeekFrom::Start(0))?;
    removed_files.read_to_string(&mut removed_files_contents)?;

    if removed_files_contents.is_empty() {
        debug("There are no host dependencies to resolve");
        return Ok(());
    }

    File::create(&host_deps)?.write_all(removed_files_contents.as_bytes())?;

    let mut call = user.run("rsync");
    call.arg("-av").arg("--ignore-missing-args").arg("--files-from").arg(&host_deps).arg("/").arg(format!("{}/", &target));
    debug(&format!("{:?}", call.get_args()));

    call.perform()?;
    Ok(())
}

pub fn init_cid_dir() -> Result<(), FlakeError> {
    if !Path::new(defaults::CONTAINER_CID_DIR).exists() {
        if !unistd::geteuid().is_root() {
            return Err(FlakeError::AccessDenied);
        }

        fs::set_permissions(defaults::CONTAINER_DIR, fs::Permissions::from_mode(0o755))?;
        fs::create_dir_all(defaults::CONTAINER_CID_DIR)?;

        // XXX: Potential CVE?
        //
        // This should not do 777. However, if this needs to be temporary,
        // then this should go to /tmp/flakes/... instead.
        fs::set_permissions(defaults::CONTAINER_CID_DIR, fs::Permissions::from_mode(0o777))?;
    }
    Ok(())
}

// Check if container with specified cid is running
pub fn container_running(cid: &str, user: User) -> Result<bool, CommandError> {
    let mut running = user.run("podman");
    running.arg("ps").arg("--format").arg("{{.ID}}");

    debug(&format!("{:?}", running.get_args()));

    for running_cid in String::from_utf8(running.perform()?.stdout).unwrap_or_default().lines() {
        if cid.starts_with(running_cid) {
            return Ok(true);
        }
    }

    Ok(false)
}

// Check if container image is present in local registry
pub fn container_image_exists(name: &str, user: User) -> Result<bool, std::io::Error> {
    let mut exists = user.run("podman");
    exists.arg("image").arg("exists").arg(name);
    debug(&format!("{:?}", exists.get_args()));
    Ok(exists.status()?.success())
}

/// Call podman pull and prune with the provided uri
pub fn pull(uri: &str, user: User) -> Result<(), FlakeError> {
    let mut pull = user.run("podman");
    pull.arg("pull").arg(uri);
    debug(&format!("{:?}", pull.get_args()));

    pull.perform()?;

    let mut prune = user.run("podman");
    prune.arg("image").arg("prune").arg("--force");
    match prune.status() {
        Ok(status) => debug(&format!("{:?}", status)),
        Err(error) => debug(&format!("{:?}", error)),
    }

    Ok(())
}

// Take the contents of the given removed_file and append it to the accumulated_file.
pub fn update_removed_files(target: &String, mut accumulated_file: &File) -> Result<(), std::io::Error> {
    let host_deps = format!("{}/{}", &target, defaults::HOST_DEPENDENCIES);
    debug(&format!("Looking up host deps from {}", host_deps));
    if Path::new(&host_deps).exists() {
        let data = fs::read_to_string(&host_deps)?;
        debug("Adding host deps...");
        debug(&String::from_utf8_lossy(data.as_bytes()));
        accumulated_file.write_all(data.as_bytes())?;
    }
    Ok(())
}

// Check if container exists according to the specified
// container_cid_file. Garbage cleanup the container_cid_file
// if no longer present. Return a true value if the container
// exists, in any other case return false.
pub fn gc_cid_file(container_cid_file: &String, user: User) -> Result<bool, FlakeError> {
    let mut exists = user.run("podman");
    exists.arg("container").arg("exists").arg(fs::read_to_string(container_cid_file)?);

    if !exists.status()?.success() {
        fs::remove_file(container_cid_file)?;
        Ok(false)
    } else {
        Ok(true)
    }
}

// Garbage collect CID files for which no container exists anymore
pub fn gc(user: User) -> Result<(), std::io::Error> {
    let mut cid_file_names: Vec<String> = Vec::new();
    let mut cid_file_count: i32 = 0;
    let paths = fs::read_dir(defaults::CONTAINER_CID_DIR)?;
    for path in paths {
        cid_file_names.push(format!("{}", path?.path().display()));
        cid_file_count += 1;
    }
    if cid_file_count > defaults::GC_THRESHOLD {
        for container_cid_file in cid_file_names {
            let _ = gc_cid_file(&container_cid_file, user);
        }
    }
    Ok(())
}
