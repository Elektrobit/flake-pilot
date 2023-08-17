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
use std::{thread, time};
use spinoff::{Spinner, spinners, Color};
use std::path::Path;
use std::process::{Command, Stdio, exit, id};
use std::env;
use std::fs;
use crate::config::{config, RuntimeSection};
use crate::defaults::{debug, is_debug};
use tempfile::{NamedTempFile, tempdir};
use std::io::{Write, SeekFrom, Seek};
use std::fs::File;
use serde::{Serialize, Deserialize};
use serde_json::{self};
use rand::Rng;

use crate::defaults;

// FireCrackerConfig represents firecracker json config
#[derive(Debug, Serialize, Deserialize)]
pub struct FireCrackerConfig {
    #[serde(rename = "boot-source")]
    pub boot_source: FireCrackerBootSource,
    pub drives: Vec<FireCrackerDrive>,
    #[serde(rename = "network-interfaces")]
    pub network_interfaces: Vec<FireCrackerNetworkInterface>,
    #[serde(rename = "machine-config")]
    pub machine_config: FireCrackerMachine,
    pub vsock: FireCrackerVsock
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FireCrackerBootSource {
    pub kernel_image_path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub initrd_path: String,
    pub boot_args: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FireCrackerDrive {
    pub drive_id: String,
    pub path_on_host: String,
    pub is_root_device: bool,
    pub is_read_only: bool,
    pub cache_type: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FireCrackerNetworkInterface {
    pub iface_id: String,
    pub guest_mac: String,
    pub host_dev_name: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FireCrackerMachine {
    pub vcpu_count: i64,
    pub mem_size_mib: i64
}
#[derive(Debug, Serialize, Deserialize)]
pub struct FireCrackerVsock {
    pub guest_cid: u32,
    pub uds_path: String
}

pub fn create(
    program_name: &String, 
) -> Vec<String> {
    /*!
    Create VM for later execution of program_name.
    The VM name and all other settings to run the program
    inside of the VM are taken from the config file(s)

    FIRECRACKER_FLAKE_DIR/
       ├── program_name.d
       │   └── other.yaml
       └── program_name.yaml

    All commandline options will be passed to the program_name
    called in the VM through the sci guestvm tool. An example
    program config file looks like the following:

    vm:
      name: name
      target_app_path: path/to/program/in/VM
      host_app_path: path/to/program/on/host

      runtime:
        # Run the VM engine as a user other than the
        # default target user root. The user may be either
        # a user name or a numeric user-ID (UID) prefixed
        # with the ‘#’ character (e.g. #0 for UID 0). The call
        # of the VM engine is performed by sudo.
        # The behavior of sudo can be controlled via the
        # file /etc/sudoers
        runas: root

        # Resume the VM from previous execution.
        # If the VM is still running, the app will be
        # executed inside of this VM instance.
        #
        # Default: false
        resume: true|false

        firecracker:
          # Currently fixed settings through app registration
          boot_args:
            - "init=/usr/sbin/sci"
            - "console=ttyS0"
            - "root=/dev/vda"
            - "acpi=off"
            - "quiet"
          mem_size_mib: 4096
          vcpu_count: 2
          cache_type: Writeback

          # Size of the VM overlay
          # If specified a new ext2 overlay filesystem image of the
          # specified size will be created and attached to the VM
          overlay_size: 20g

          # Path to rootfs image done by app registration
          rootfs_image_path: /var/lib/firecracker/images/NAME/rootfs

          # Path to kernel image done by app registration
          kernel_image_path: /var/lib/firecracker/images/NAME/kernel

          # Optional path to initrd image done by app registration
          initrd_path: /var/lib/firecracker/images/NAME/initrd

      Calling this method returns a vector including a placeholder
      for the later VM process ID and and the name of
      the VM ID file.
    !*/
    let mut result: Vec<String> = Vec::new();

    // setup VM ID file name
    let vm_id_file = get_meta_file_name(
        program_name, defaults::FIRECRACKER_VMID_DIR, "vmid"
    );

    // get flake config sections
    let RuntimeSection { runas, resume, firecracker: engine_section, .. } = config().runtime();

    // check for includes
    let tar_includes = config().tars();
    let has_includes = !tar_includes.is_empty();

    // Make sure meta dirs exists
    init_meta_dirs();

    // Check early return condition
    if Path::new(&vm_id_file).exists() && gc_meta_files(&vm_id_file, runas, program_name, resume) && resume {
        // VM exists
        // report ID value and its ID file name
        match fs::read_to_string(&vm_id_file) {
            Ok(vmid) => {
                result.push(vmid);
            },
            Err(error) => {
                // vmid file exists but could not be read
                panic!("Error reading VMID: {:?}", error);
            }
        }
        result.push(vm_id_file);
        return result;
    }

    // Garbage collect occasionally
    gc(runas, program_name);

    // Sanity check
    if Path::new(&vm_id_file).exists() {
        // we are about to create a VM for which a
        // vmid file already exists.
        error!(
            "VM ID in use by another instance, consider @NAME argument"
        );
        exit(1)
    }

    // Setup VM...
    let spinner = Spinner::new_with_stream(
        spinners::Line, "Launching flake...", Color::Yellow, spinoff::Streams::Stderr
    );

    // Create initial vm_id_file with process ID set to 0
    match std::fs::File::create(&vm_id_file) {
        Ok(mut vm_id_fd) => {
            let vm_id = "0";
            match vm_id_fd.write_all(vm_id.as_bytes()) {
                Ok(_) => {
                    result.push(vm_id.to_string());
                    result.push(vm_id_file);
                },
                Err(error) => {
                    panic!("Failed to write to file {}: {}", vm_id_file, error)
                }
            }
        },
        Err(error) => {
            panic!("Failed to open {}: {}", vm_id_file, error)
        }
    }

    // Setup root overlay if configured
    let mut provision_ok = false;
    let vm_overlay_file = get_meta_file_name(
        program_name, defaults::FIRECRACKER_OVERLAY_DIR, "ext2"
    );
    if let Some(overlay_size) = engine_section.overlay_size {
        let overlay_size = overlay_size.as_u64();
        if !Path::new(&vm_overlay_file).exists() || !resume {
            match std::fs::File::create(&vm_overlay_file) {
                Ok(mut vm_overlay_file_fd) => {
                    match vm_overlay_file_fd.seek(
                        SeekFrom::Start(overlay_size - 1)
                    ) {
                        Ok(_) => {
                            match vm_overlay_file_fd.write_all(&[0]) {
                                Ok(_) => { },
                                Err(error) => {
                                    panic!("Write failed with: {}", error)
                                }
                            }
                        },
                        Err(error) => {
                            panic!("No space left on device: {}", error)
                        }
                    }
                },
                Err(error) => {
                    panic!("Failed to create overlay image: {}", error);
                }
            }
            // Create filesystem
            let mut mkfs = Command::new("sudo");
            if let Some(runas) = runas {
                mkfs.arg("--user").arg(runas);
            }
            mkfs
                .arg("mkfs.ext2")
                .arg("-F")
                .arg(&vm_overlay_file);
            debug(&format!("sudo {:?}", mkfs.get_args()));
            match mkfs.output() {
                Ok(output) => {
                    if ! output.status.success() {
                        panic!(
                            "Failed to create overlay filesystem: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                },
                Err(error) => {
                    panic!("Failed to execute mkfs {:?}", error)
                }
            }
            provision_ok = true;
        }
    }

    // Provision VM
    if provision_ok {
        let vm_image_file = engine_section.rootfs_image_path;
        match tempdir() {
            Ok(tmp_dir) => {
                let vm_mount_point = mount_vm(
                    tmp_dir.path().to_str().unwrap(),
                    &vm_image_file,
                    &vm_overlay_file,
                    Some("root")
                );
                if ! vm_mount_point.is_empty() {
                    // Handle includes
                    if has_includes {
                        debug("Syncing includes...");
                        provision_ok = sync_includes(
                            &vm_mount_point, Some("root")
                        );
                    }
                } else {
                    provision_ok = false
                }
                umount_vm(
                    tmp_dir.path().to_str().unwrap(),
                    Some("root")
                );
            },
            Err(error) => {
                error!("Failed to create temporary file: {}", error);
                provision_ok = false
            }
        }
        if ! provision_ok {
            spinner.fail("Flake launch has failed");
            panic!("Failed to provision VM")
        }
    }

    spinner.success("Launching flake");
    result
}

pub fn start(
    program_name: &String, vm: Vec<String>
) {
    /*!
    Start VM with the given VM ID

    firecracker-pilot exits with the return code from firecracker
    after this function
    !*/
    let RuntimeSection { runas, resume, .. } = config().runtime();
    let vmid = &vm[0];
    let vm_id_file = &vm[1];

    let status_code;
    let mut is_running: bool = false;
    let mut is_blocking: bool = true;

    if vm_running(vmid, runas) {
        is_running = true;
    }

    if is_running {
        // 1. Execute app in running VM
        status_code = execute_command_at_instance(
            program_name, runas, get_exec_port()
        );
    } else {
        match NamedTempFile::new() {
            Ok(firecracker_config) => {
                create_firecracker_config(
                    program_name, &firecracker_config
                );
                if resume {
                    // 2. Startup resume type VM and execute app
                    is_blocking = false;
                    call_instance(
                        &firecracker_config, vm_id_file, runas, is_blocking
                    );
                    status_code = execute_command_at_instance(
                        program_name, runas, get_exec_port()
                    );
                } else {
                    // 3. Startup VM and execute app
                    status_code = call_instance(
                        &firecracker_config, vm_id_file, runas, is_blocking
                    );
                }
            },
            Err(error) => {
                panic!("Failed to create temporary file: {}", error)
            }
        }
    }
    exit(status_code)
}

pub fn call_instance(
    config_file: &NamedTempFile, vm_id_file: &String,
    user: Option<&str>, is_blocking: bool
) -> i32 {
    /*!
    Run firecracker with specified configuration
    !*/
    let mut status_code = 0;

    let mut firecracker = Command::new("sudo");
    if let Some(user) = user {
        firecracker.arg("--user").arg(user);
    }
    if ! is_debug() {
        firecracker.stderr(Stdio::null());
    }
    if ! is_debug() && ! is_blocking {
        firecracker
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
    }
    firecracker
        .arg("firecracker")
        .arg("--no-api")
        .arg("--id")
        .arg(id().to_string())
        .arg("--config-file")
        .arg(config_file.path());
    debug(&format!("sudo {:?}", firecracker.get_args()));
    match firecracker.spawn() {
        Ok(mut child) => {
            let pid = child.id();
            debug(&format!("PID {}", pid));
            match std::fs::File::create(vm_id_file) {
                Ok(mut vm_id_fd) => {
                    match vm_id_fd.write_all(pid.to_string().as_bytes()) {
                        Ok(_) => { },
                        Err(error) => {
                            panic!(
                                "Failed to write to file {}: {}",
                                vm_id_file, error
                            )
                        }
                    }
                },
                Err(error) => {
                    panic!("Failed to open {}: {}", vm_id_file, error)
                }
            }
            if is_blocking {
                match child.wait() {
                    Ok(ecode) => {
                        if ecode.code().is_some() {
                            status_code = ecode.code().unwrap()
                        }
                    },
                    Err(error) => {
                        panic!("firecracker failed with: {}", error);
                    }
                }
            }
        },
        Err(error) => {
            panic!("Failed to execute firecracker: {:?}", error)
        }
    }
    status_code
}

pub fn get_exec_port() -> u32 {
    /*!
    Create random execution port
    !*/
    let mut random = rand::thread_rng();
    // FIXME: A more stable version
    // should check for already running socket connections
    // and if the same number is used for an already running one
    // another rand should be called
    
    random.gen_range(49200..60000)
}

pub fn check_connected(program_name: &String, user: Option<&str>) -> i32 {
    /*!
    Check if instance connection is OK
    !*/
    let mut status_code;
    let mut retry_count = 0;
    let vsock_uds_path = format!(
        "/run/sci_cmd_{}.sock", get_meta_name(program_name)
    );
    loop {
        if retry_count == defaults::RETRIES {
            debug("Max retries for VM connection check exceeded");
            status_code = 1;
            return status_code
        }
        let mut vm_command = Command::new("sudo");
        if let Some(user) = user {
            vm_command.arg("--user").arg(user);
        }
        vm_command
            .arg("bash")
            .arg("-c")
            .arg(&format!(
                "echo -e 'CONNECT {}'|{} UNIX-CONNECT:{} -",
                defaults::VM_PORT,
                defaults::SOCAT,
                vsock_uds_path
            ));
        debug(&format!("sudo {:?}", vm_command.get_args()));
        match vm_command.output() {
            Ok(output) => {
                if String::from_utf8_lossy(&output.stdout).starts_with("OK") {
                    status_code = 0
                } else {
                    status_code = 1
                }
            },
            Err(error) => {
                error!("UNIX-CONNECT failed with: {:?}", error);
                status_code = 1
            }
        }
        if status_code == 0 {
            // connection OK
            break
        } else {
            // VM not ready for connections
            let some_time = time::Duration::from_millis(
                defaults::VM_WAIT_TIMEOUT_MSEC
            );
            thread::sleep(some_time);
        }
        retry_count += 1
    }
    status_code
}

pub fn send_command_to_instance(
    program_name: &String, user: Option<&str>, exec_port: u32
) -> i32 {
    /*!
    Send command to the VM via a vsock
    !*/
    let mut status_code;
    let mut retry_count = 0;
    let run = get_run_cmdline(program_name, false);
    let vsock_uds_path = format!(
        "/run/sci_cmd_{}.sock", get_meta_name(program_name)
    );
    loop {
        if retry_count == defaults::RETRIES {
            debug("Max retries for VM command transfer exceeded");
            status_code = 1;
            return status_code
        }
        let mut vm_command = Command::new("sudo");
        if let Some(user) = user {
            vm_command.arg("--user").arg(user);
        }
        vm_command
            .arg("bash")
            .arg("-c")
            .arg(&format!(
                "echo -e 'CONNECT {}\n{} {}\n'|{} UNIX-CONNECT:{} -",
                defaults::VM_PORT,
                run.join(" "),
                exec_port,
                defaults::SOCAT,
                vsock_uds_path
            ));
        debug(&format!("sudo {:?}", vm_command.get_args()));
        match vm_command.output() {
            Ok(output) => {
                if String::from_utf8_lossy(&output.stdout).starts_with("OK") {
                    status_code = 0
                } else {
                    status_code = 1
                }
            },
            Err(error) => {
                error!("UNIX-CONNECT failed with: {:?}", error);
                status_code = 1
            }
        }
        if status_code == 0 {
            // command transfered
            break
        } else {
            // VM not ready for connections
            let some_time = time::Duration::from_millis(
                defaults::VM_WAIT_TIMEOUT_MSEC
            );
            thread::sleep(some_time);
        }
        retry_count += 1
    }
    status_code
}

pub fn execute_command_at_instance(
    program_name: &String, user: Option<&str>, exec_port: u32
) -> i32 {
    /*!
    Send command to a vsoc connected to a running instance
    !*/
    let mut status_code;
    let mut retry_count = 0;
    let vsock_uds_path = format!(
        "/run/sci_cmd_{}.sock", get_meta_name(program_name)
    );

    // wait for UDS socket to appear
    loop {
        if retry_count == defaults::RETRIES {
            debug("Max retries for UDS socket lookup exceeded");
            status_code = 1;
            return status_code
        }
        if Path::new(&vsock_uds_path).exists() {
            break
        }
        let some_time = time::Duration::from_millis(100);
        thread::sleep(some_time);
        retry_count += 1
    }

    // make sure instance can be contacted
    if check_connected(program_name, user) != 0 {
        return 1
    }

    // spawn the listener and wait for sci to run the command
    let mut vm_exec = Command::new("sudo");
    if let Some(user) = user {
        vm_exec.arg("--user").arg(user);
    }
    vm_exec
        .arg(defaults::SOCAT)
        .arg("-t")
        .arg("0")
        .arg("-")
        .arg(
            &format!("UNIX-LISTEN:{}_{}",
            vsock_uds_path, exec_port
        ));
    debug(&format!("sudo {:?}", vm_exec.get_args()));
    match vm_exec.spawn() {
        Ok(mut child) => {
            status_code = send_command_to_instance(
                program_name, user, exec_port
            );
            match child.wait() {
                Ok(ecode) => {
                    status_code = ecode.code().unwrap();
                },
                Err(error) => {
                    error!("command failed with: {}", error);
                }
            }
        },
        Err(error) => {
            error!("UNIX-LISTEN failed with: {:?}", error);
            status_code = 1
        }
    }
    status_code
}

pub fn create_firecracker_config(
    program_name: &String,
    config_file: &NamedTempFile
) {
    /*!
    Create json config to call firecracker
    !*/
    match std::fs::File::open(defaults::FIRECRACKER_TEMPLATE) {
        Ok(template) => {
            match serde_json::from_reader::<File, FireCrackerConfig>(template) {
                Ok(mut firecracker_config) => {
                    let mut boot_args: Vec<String> = Vec::new();
                    let RuntimeSection { resume, firecracker: engine_section, .. } = config().runtime();

                    // set kernel_image_path
                    firecracker_config.boot_source.kernel_image_path = engine_section.kernel_image_path.to_owned();

                    // set initrd_path
                    if let Some(initrd_path) = engine_section.initrd_path {
                        firecracker_config.boot_source.initrd_path = initrd_path.to_owned();
                    }

                    // setup run commandline for the command call
                    let run = get_run_cmdline(
                        program_name, true
                    );

                    // set boot_args
                    if is_debug() {
                        boot_args.push("PILOT_DEBUG=1".to_string());
                    }
                    if engine_section.overlay_size.is_some() {
                        boot_args.push("overlay_root=/dev/vdb".to_string());
                    }
                    for boot_option in engine_section.boot_args
                    {
                        if resume
                            && ! is_debug()
                            && boot_option.starts_with("console=")
                        {
                            // in resume mode the communication is handled
                            // through vsocks. Thus we don't need a serial
                            // console and only provide one in debug mode
                            boot_args.push("console=".to_string());
                        } else {
                            boot_args.push(boot_option.to_owned());
                        }
                        }
                    if ! firecracker_config.boot_source.boot_args.is_empty() {
                        firecracker_config.boot_source.boot_args.push(' ');
                    }
                    firecracker_config.boot_source.boot_args.push_str(
                        &boot_args.join(" ")
                    );
                    if resume {
                        firecracker_config.boot_source.boot_args.push_str(
                            " run=vsock"
                        )
                    } else {
                        firecracker_config.boot_source.boot_args.push_str(
                            &format!(" run=\"{}\"", run.join(" "))
                        )
                    }

                    // set path_on_host for rootfs
                    firecracker_config.drives[0].path_on_host = engine_section.rootfs_image_path.to_owned();

                    // set drive section for overlay
                    if engine_section.overlay_size.is_some() {
                        let vm_overlay_file = get_meta_file_name(
                            program_name,
                            defaults::FIRECRACKER_OVERLAY_DIR,
                            "ext2"
                        );

                        let cache_type = engine_section.cache_type.unwrap_or_default().to_string();

                        let drive = FireCrackerDrive {
                            drive_id: "overlay".to_string(),
                            path_on_host: vm_overlay_file,
                            is_root_device: false,
                            is_read_only: false,
                            cache_type
                        };
                        firecracker_config.drives.push(drive);
                    }

                    // set tap device name
                    firecracker_config.network_interfaces[0].host_dev_name =
                        format!("tap-{}", get_meta_name(program_name));

                    // set vsock name
                    firecracker_config.vsock.guest_cid = defaults::VM_CID;
                    firecracker_config.vsock.uds_path = format!(
                        "/run/sci_cmd_{}.sock", get_meta_name(program_name)
                    );

                    // set mem_size_mib
                    if let Some(mem_size_mib) = engine_section.mem_size_mib {
                        firecracker_config.machine_config.mem_size_mib = mem_size_mib
                    }

                    // set vcpu_count
                    if let Some(vcpu_count) = engine_section.vcpu_count {
                        firecracker_config.machine_config.vcpu_count = vcpu_count;
                    }

                    debug(&serde_json::to_string(&firecracker_config).unwrap());
                    serde_json::to_writer(
                        config_file, &firecracker_config
                    ).unwrap();
                },
                Err(error) => {
                    panic!("Failed to import config template: {}", error);
                }
            }
        },
        Err(error) => {
            panic!(
                "Failed to open {}: {}", defaults::FIRECRACKER_TEMPLATE, error
            )
        }
    }
}

pub fn get_target_app_path(
    program_name: &str, 
) -> String {
    /*!
    setup application command path name

    This is either the program name specified at registration
    time or the configured target application from the flake
    configuration file
    !*/
    config().vm.target_app_path.unwrap_or(program_name).to_owned()

}

pub fn init_meta_dirs() {
    let mut meta_dirs: Vec<&str> = Vec::new();
    meta_dirs.push(defaults::FIRECRACKER_OVERLAY_DIR);
    meta_dirs.push(defaults::FIRECRACKER_VMID_DIR);
    for meta_dir in meta_dirs {
        if ! Path::new(meta_dir).is_dir() && ! mkdir(meta_dir, "777", Some("root")) {
            panic!("Failed to create {}", meta_dir);
        }
    }
}

pub fn get_run_cmdline(
    program_name: &str,
    quote_for_kernel_cmdline: bool
) -> Vec<String> {
    /*!
    setup run commandline for the command call
    !*/
    let args: Vec<String> = env::args().collect();
    let mut run: Vec<String> = Vec::new();
    let target_app_path = get_target_app_path(
        program_name
    );
    run.push(target_app_path);
    for arg in &args[1..] {
        debug(&format!("Got Argument: {}", arg));
        if ! arg.starts_with('@') {
            if quote_for_kernel_cmdline {
                run.push(arg.replace('-', "\\-").to_string());
            } else {
                run.push(arg.to_string());
            }
        }
    }
    run
}

pub fn vm_running(vmid: &String, user: Option<&str>) -> bool {
    /*!
    Check if VM with specified vmid is running
    !*/
    let mut running_status = false;
    if vmid == "0" {
        return running_status
    }
    let mut running = Command::new("sudo");
    if let Some(user) = user {
        running.arg("--user").arg(user);
    }
    running.arg("kill").arg("-0").arg(vmid);
    debug(&format!("{:?}", running.get_args()));
    match running.output() {
        Ok(output) => {
            if output.status.code().unwrap() == 0 {
                running_status = true
            }
        },
        Err(error) => {
            panic!("Failed to execute kill -0: {:?}", error)
        }
    }
    running_status
}

pub fn get_meta_file_name(
    program_name: &String, target_dir: &str, extension: &str
) -> String {
    /*!
    Construct meta data file name from given program name
    !*/
    let meta_file = format!(
        "{}/{}.{}", target_dir, get_meta_name(program_name), extension
    );
    meta_file
}

pub fn get_meta_name(program_name: &String) -> String {
    /*!
    Construct meta data basename from given program name
    !*/
    let args: Vec<String> = env::args().collect();
    let mut meta_file = program_name.to_string();
    for arg in &args[1..] {
        if arg.starts_with('@') {
            // The special @NAME argument is not passed to the
            // actual call and can be used to run different VM
            // instances for the same application
            meta_file = format!("{}{}", meta_file, arg);
        }
    }
    meta_file
}

pub fn gc_meta_files(
    vm_id_file: &String, user: Option<&str>, program_name: &String, resume: bool
) -> bool {
    /*!
    Check if VM exists according to the specified
    vm_id_file. Garbage cleanup the vm_id_file and the vsock socket
    if no longer present. Return a true value if the VM
    exists, in any other case return false.
    !*/
    let mut vmid_status = false;
    match fs::read_to_string(vm_id_file) {
        Ok(vmid) => {
            if ! vm_running(&vmid, user) {
                debug(&format!("Deleting {}", vm_id_file));
                match fs::remove_file(vm_id_file) {
                    Ok(_) => { },
                    Err(error) => {
                        error!("Failed to remove VMID: {:?}", error)
                    }
                }
                let vsock_uds_path = format!(
                    "/run/sci_cmd_{}.sock", get_meta_name(program_name)
                );
                if Path::new(&vsock_uds_path).exists() {
                    debug(&format!("Deleting {}", vsock_uds_path));
                    delete_file(&vsock_uds_path, user);
                }
                let vm_overlay_file = format!(
                    "{}/{}", defaults::FIRECRACKER_OVERLAY_DIR,
                    Path::new(&vm_id_file)
                        .file_name().unwrap().to_str().unwrap()
                        .replace(".vmid", ".ext2")
                );
                if Path::new(&vm_overlay_file).exists() && ! resume {
                    debug(&format!("Deleting {}", vm_overlay_file));
                    match fs::remove_file(&vm_overlay_file) {
                        Ok(_) => { },
                        Err(error) => {
                            error!("Failed to remove VMID: {:?}", error)
                        }
                    }
                }
            } else {
                vmid_status = true
            }
        },
        Err(error) => {
            error!("Error reading VMID: {:?}", error);
        }
    }
    vmid_status
}

pub fn gc(user: Option<&str>, program_name: &String) {
    /*!
    Garbage collect VMID files for which no VM exists anymore
    !*/
    let mut vmid_file_names: Vec<String> = Vec::new();
    let mut vmid_file_count: i32 = 0;
    let paths = fs::read_dir(defaults::FIRECRACKER_VMID_DIR).unwrap();
    for path in paths {
        vmid_file_names.push(format!("{}", path.unwrap().path().display()));
        vmid_file_count += 1;
    }
    if vmid_file_count <= defaults::GC_THRESHOLD {
        return
    }
    for vm_id_file in vmid_file_names {
        // collective garbage collect but do not delete overlay
        // images as they might be re-used for resume type instances.
        // The cleanup of overlay images from resume type instances
        // must be done by an explicit user action to avoid deleting
        // user data in overlay images eventually preserved for later.
        gc_meta_files(&vm_id_file, user, program_name, true);
    }
}

pub fn delete_file(filename: &String, user: Option<&str>) -> bool {
    /*!
    Delete file via sudo
    !*/
    let mut call = Command::new("sudo");
    if let Some(user) = user {
        call.arg("--user").arg(user);
    }
    call.arg("rm").arg("-f").arg(filename);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to rm: {}: {:?}", filename, error);
            return false
        }
    }
    true
}

pub fn sync_includes(
    target: &str, user: Option<&str>
) -> bool {
    /*!
    Sync custom include data to target path
    !*/
    let tar_includes = config().tars();
    let mut status_code = 0;
    for tar in tar_includes {
        debug(&format!("Adding tar include: [{}]", tar));
        let mut call = Command::new("sudo");
        if let Some(user) = user {
            call.arg("--user").arg(user);
        }
        call.arg("tar")
            .arg("-C").arg(target)
            .arg("-xf").arg(tar);
        debug(&format!("{:?}", call.get_args()));
        match call.output() {
            Ok(output) => {
                debug(&String::from_utf8_lossy(&output.stdout));
                debug(&String::from_utf8_lossy(&output.stderr));
                status_code = output.status.code().unwrap();
            },
            Err(error) => {
                panic!("Failed to execute tar: {:?}", error)
            }
        }
    }
    if status_code == 0 {
        return true
    }
    false
}

pub fn mount_vm(
    sub_dir: &str, rootfs_image_path: &str,
    overlay_path: &str, user: Option<&str>
) -> String {
    /*!
    Mount VM with overlay below given sub_dir
    !*/
    let failed = "".to_string();
    // 1. create overlay image mount structure
    for image_dir in vec![
        defaults::IMAGE_ROOT,
        defaults::IMAGE_OVERLAY
    ].iter() {
        let dir_path = format!("{}/{}", sub_dir, image_dir);
        if ! Path::new(&dir_path).exists() {
            match fs::create_dir_all(&dir_path) {
                Ok(_) => { },
                Err(error) => {
                    error!("Error creating directory {}: {}", dir_path, error);
                    return failed
                }
            }
        }
    }
    // 2. mount VM image
    let image_mount_point = format!(
        "{}/{}", sub_dir, defaults::IMAGE_ROOT
    );
    let mut mount_image = Command::new("sudo");
    if let Some(user) = user {
        mount_image.arg("--user").arg(user);
    }
    mount_image
        .arg("mount")
        .arg(rootfs_image_path)
        .arg(&image_mount_point);
    debug(&format!("{:?}", mount_image.get_args()));
    match mount_image.output() {
        Ok(output) => {
            if ! output.status.success() {
                error!(
                    "Failed to mount VM image: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return failed
            }
        },
        Err(error) => {
            error!("Failed to execute mount: {:?}", error);
            return failed
        }
    }
    // 3. mount Overlay image
    let overlay_mount_point = format!(
        "{}/{}", sub_dir, defaults::IMAGE_OVERLAY
    );
    let mut mount_overlay = Command::new("sudo");
    if let Some(user) = user {
        mount_overlay.arg("--user").arg(user);
    }
    mount_overlay
        .arg("mount")
        .arg(overlay_path)
        .arg(&overlay_mount_point);
    debug(&format!("{:?}", mount_overlay.get_args()));
    match mount_overlay.output() {
        Ok(output) => {
            if ! output.status.success() {
                error!(
                    "Failed to mount VM image: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return failed
            }
        },
        Err(error) => {
            error!("Failed to execute mount: {:?}", error);
            return failed
        }
    }
    // 4. mount as overlay
    for overlay_dir in vec![
        defaults::OVERLAY_ROOT,
        defaults::OVERLAY_UPPER,
        defaults::OVERLAY_WORK
    ].iter() {
        let dir_path = format!("{}/{}", sub_dir, overlay_dir);
        if ! Path::new(&dir_path).exists() && ! mkdir(&dir_path, "755", Some("root")) {
            return failed
        }
    }
    let root_mount_point = format!("{}/{}", sub_dir, defaults::OVERLAY_ROOT);
    let mut mount_overlay = Command::new("sudo");
    if let Some(user) = user {
        mount_overlay.arg("--user").arg(user);
    }
    mount_overlay
        .arg("mount")
        .arg("-t")
        .arg("overlay")
        .arg("overlayfs")
        .arg("-o")
        .arg(format!("lowerdir={},upperdir={}/{},workdir={}/{}",
            &image_mount_point,
            sub_dir, defaults::OVERLAY_UPPER,
            sub_dir, defaults::OVERLAY_WORK
        ))
        .arg(&root_mount_point);
    debug(&format!("{:?}", mount_overlay.get_args()));
    match mount_overlay.output() {
        Ok(output) => {
            if ! output.status.success() {
                error!(
                    "Failed to overlay mount VM image: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                return failed
            }
        },
        Err(error) => {
            error!("Failed to execute mount: {:?}", error);
            return failed
        }
    }
    root_mount_point
}

pub fn umount_vm(sub_dir: &str, user: Option<&str>) -> bool {
    /*!
    Umount VM image
    !*/
    let mut status_code = 0;
    for mount_point in vec![
        defaults::OVERLAY_ROOT,
        defaults::IMAGE_OVERLAY,
        defaults::IMAGE_ROOT,
    ].iter() {
        let mut umount = Command::new("sudo");
        umount.stderr(Stdio::null());
        umount.stdout(Stdio::null());
        if let Some(user) = user {
            umount.arg("--user").arg(user);
        }
        umount.arg("umount").arg(format!("{}/{}", &sub_dir, &mount_point));
        debug(&format!("{:?}", umount.get_args()));
        match umount.status() {
            Ok(status) => {
                status_code += status.code().unwrap();
            },
            Err(error) => {
                error!("Failed to execute umount: {:?}", error)
            }
        }
    }
    if status_code > 0 {
        return false
    }
    true
}

pub fn mkdir(dirname: &str, mode: &str, user: Option<&str>) -> bool {
    /*!
    Make directory via sudo
    !*/
    let mut call = Command::new("sudo");
    if let Some(user) = user {
        call.arg("--user").arg(user);
    }
    call.arg("mkdir").arg("-p").arg("-m").arg(mode).arg(dirname);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to mkdir: {}: {:?}", dirname, error);
            return false
        }
    }
    true
}
