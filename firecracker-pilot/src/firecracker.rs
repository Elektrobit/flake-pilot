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
use spinoff::{Spinner, spinners, Color};
use yaml_rust::Yaml;
use std::path::Path;
use std::process::{Command, Stdio, exit, id};
use std::os::unix::fs::PermissionsExt;
use std::env;
use std::fs;
use crate::defaults::{debug, is_debug};
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs::File;
use serde::{Serialize, Deserialize};
use serde_json::{self};

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
    pub machine_config: FireCrackerMachine
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

pub fn create(
    program_name: &String, runtime_config: &Vec<Yaml>
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
    let vm_section = &runtime_config[0]["vm"];
    let runtime_section = &vm_section["runtime"];
    let engine_section = &runtime_section["firecracker"];

    // setup VM operation mode
    let mut runas = String::new();
    let mut resume: bool = false;

    if ! runtime_section.as_hash().is_none() {
        if ! &runtime_section["runas"].as_str().is_none() {
            runas.push_str(&runtime_section["runas"].as_str().unwrap());
        }
        if ! &runtime_section["resume"].as_bool().is_none() {
            resume = runtime_section["resume"].as_bool().unwrap();
        }
    }

    // Make sure meta dirs exists
    init_meta_dirs();

    // Check early return condition
    if Path::new(&vm_id_file).exists() &&
        gc_meta_files(&vm_id_file, &runas, resume)
    {
        if resume {
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
    }

    // Garbage collect occasionally
    gc(&runas);

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
    let spinner = Spinner::new(
        spinners::Line, "Launching flake...", Color::Yellow
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
    if ! &engine_section["overlay_size"].as_str().is_none() {
        let vm_overlay_file = get_meta_file_name(
            program_name, defaults::FIRECRACKER_OVERLAY_DIR, "ext2"
        );
        if ! Path::new(&vm_overlay_file).exists() || ! resume {
            let mut qemu_img = Command::new("sudo");
            if ! runas.is_empty() {
                qemu_img.arg("--user").arg(&runas);
            }
            qemu_img
                .arg("qemu-img")
                .arg("create")
                .arg(&vm_overlay_file)
                .arg(&engine_section["overlay_size"].as_str().unwrap());
            debug(&format!("sudo {:?}", qemu_img.get_args()));
            match qemu_img.output() {
                Ok(output) => {
                    if ! output.status.success() {
                        panic!(
                            "Failed to create overlay image: {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                },
                Err(error) => {
                    panic!("Failed to execute qemu-img {:?}", error)
                }
            }
            let mut mkfs = Command::new("sudo");
            if ! runas.is_empty() {
                mkfs.arg("--user").arg(&runas);
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
        }
    }

    // NOTE:
    // Provision data prior start
    // The provision code is currently missing in this pilot.
    // At this point in the code the handling for:
    //
    // - include
    // - layers
    // - delta
    //
    // need to be added. Provisioning is only possible if
    // an overlay is configured. The provision code needs to
    // rsync the data into the overlay. The overlay is handled
    // as an overlayfs which requires the provision code to
    // mount the overlay using overlayfs and sync the data into
    // the overlayfs mount point

    spinner.success("Launching flake");
    return result;
}

pub fn start(
    program_name: &String, runtime_config: &Vec<Yaml>, vm: Vec<String>
) {
    /*!
    Start VM with the given VM ID

    firecracker-pilot exits with the return code from firecracker
    after this function
    !*/
    let vm_section = &runtime_config[0]["vm"];
    let runtime_section = &vm_section["runtime"];
    let vmid = &vm[0];
    let vm_id_file = &vm[1];

    let status_code;
    let mut resume: bool = false;
    let mut is_running: bool = false;
    let mut runas = String::new();

    if ! runtime_section.as_hash().is_none() {
        if ! &runtime_section["resume"].as_bool().is_none() {
            resume = runtime_section["resume"].as_bool().unwrap();
        }
        if ! &runtime_section["runas"].as_str().is_none() {
            runas.push_str(&runtime_section["runas"].as_str().unwrap());
        }
    }

    if vm_running(&vmid, &runas) {
        is_running = true;
    }

    if is_running {
        // 1. Execute app in running VM
        status_code = send_command_to_instance();
    } else {
        match NamedTempFile::new() {
            Ok(firecracker_config) => {
                create_firecracker_config(
                    &program_name, &runtime_config, &firecracker_config
                );
                if resume {
                    // 2. Startup resume type VM and execute app
                    call_instance(
                        &firecracker_config, vm_id_file, &runas
                    );
                    status_code = send_command_to_instance();
                } else {
                    // 3. Startup VM and execute app
                    status_code = call_instance(
                        &firecracker_config, vm_id_file, &runas
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
    config_file: &NamedTempFile, vm_id_file: &String, user: &String
) -> i32 {
    /*!
    Run firecracker with specified configuration
    !*/
    let status_code;

    let mut firecracker = Command::new("sudo");
    if ! user.is_empty() {
        firecracker.arg("--user").arg(&user);
    }
    if ! is_debug() {
        firecracker.stderr(Stdio::null());
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
            match child.wait() {
                Ok(ecode) => {
                    status_code = ecode.code().unwrap();
                },
                Err(error) => {
                    panic!("firecracker failed with: {}", error);
                }
            }
        },
        Err(error) => {
            panic!("Failed to execute firecracker: {:?}", error)
        }
    }
    status_code
}

pub fn send_command_to_instance() -> i32 {
    /*!
    Send command to a vsoc connected to a running instance
    !*/
    let status_code = 0;
    // NOTE:
    // This requires the vsoc and reading from it
    // implementation in sci first
    status_code
}

pub fn create_firecracker_config(
    program_name: &String, runtime_config: &Vec<Yaml>,
    config_file: &NamedTempFile
) {
    /*!
    Create json config to call firecracker
    !*/
    match std::fs::File::open(defaults::FIRECRACKER_TEMPLATE) {
        Ok(template) => {
            match serde_json::from_reader::<File, FireCrackerConfig>(template) {
                Ok(mut firecracker_config) => {
                    let args: Vec<String> = env::args().collect();
                    let mut run: Vec<String> = Vec::new();
                    let mut boot_args: Vec<String> = Vec::new();
                    let vm_section = &runtime_config[0]["vm"];
                    let runtime_section = &vm_section["runtime"];
                    let engine_section = &runtime_section["firecracker"];
                    let mut resume: bool = false;

                    // check for resume mode
                    if ! runtime_section.as_hash().is_none() {
                        if ! &runtime_section["resume"].as_bool().is_none() {
                            resume = runtime_section["resume"]
                                .as_bool().unwrap();
                        }
                    }

                    // set kernel_image_path
                    firecracker_config.boot_source.kernel_image_path =
                        engine_section["kernel_image_path"]
                            .as_str().unwrap().to_string();

                    // set initrd_path
                    if ! engine_section["initrd_path"].as_str().is_none() {
                        firecracker_config.boot_source.initrd_path =
                            engine_section["initrd_path"]
                                .as_str().unwrap().to_string()
                    }

                    // setup run commandline for the command call
                    let target_app_path = get_target_app_path(
                        &program_name, &runtime_config
                    );
                    run.push(target_app_path);
                    for arg in &args[1..] {
                        if ! arg.starts_with("@") {
                            run.push(arg.replace("-", "\\-").to_string());
                        }
                    }

                    // set boot_args
                    if is_debug() {
                        boot_args.push("PILOT_DEBUG=1".to_string());
                    }
                    if resume {
                        boot_args.push("sci_resume=1".to_string());
                    }
                    if ! &engine_section["overlay_size"].as_str().is_none() {
                        boot_args.push("overlay_root=/dev/vdb".to_string());
                    }
                    if ! engine_section["boot_args"].as_vec().is_none() {
                        for boot_arg in
                            engine_section["boot_args"].as_vec().unwrap()
                        {
                            boot_args.push(
                                boot_arg.as_str().unwrap().to_string()
                            );
                        }
                    }
                    firecracker_config.boot_source.boot_args =
                        format!(
                            "run=\"{}\" {} {}",
                            run.join(" "),
                            firecracker_config.boot_source.boot_args,
                            boot_args.join(" ")
                        );

                    // set path_on_host for rootfs
                    firecracker_config.drives[0].path_on_host =
                        engine_section["rootfs_image_path"]
                        .as_str().unwrap().to_string();

                    // set drive section for overlay
                    if ! &engine_section["overlay_size"].as_str().is_none() {
                        let mut cache_type = "Writeback".to_string();
                        let vm_overlay_file = get_meta_file_name(
                            program_name,
                            defaults::FIRECRACKER_OVERLAY_DIR,
                            "ext2"
                        );
                        if ! &engine_section["cache_type"].as_str().is_none() {
                            cache_type = engine_section["cache_type"]
                                .as_str().unwrap().to_string();
                        }
                        let drive = FireCrackerDrive {
                            drive_id: "overlay".to_string(),
                            path_on_host: vm_overlay_file,
                            is_root_device: false,
                            is_read_only: false,
                            cache_type: cache_type
                        };
                        firecracker_config.drives.push(drive);
                    }

                    // set mem_size_mib
                    if ! &engine_section["mem_size_mib"].as_i64().is_none() {
                        firecracker_config.machine_config.mem_size_mib =
                            engine_section["mem_size_mib"].as_i64().unwrap();
                    }

                    // set vcpu_count
                    if ! &engine_section["vcpu_count"].as_i64().is_none() {
                        firecracker_config.machine_config.vcpu_count =
                            engine_section["vcpu_count"].as_i64().unwrap();
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
    program_name: &String, runtime_config: &Vec<Yaml>
) -> String {
    /*!
    setup application command path name

    This is either the program name specified at registration
    time or the configured target application from the flake
    configuration file
    !*/
    let mut target_app_path = String::new();
    let vm_section = &runtime_config[0]["vm"];
    if ! vm_section["target_app_path"].as_str().is_none() {
        target_app_path.push_str(
            vm_section["target_app_path"].as_str().unwrap()
        )
    } else {
        target_app_path.push_str(program_name.as_str())
    }
    return target_app_path
}

pub fn init_meta_dirs() {
    let mut meta_dirs: Vec<&str> = Vec::new();
    meta_dirs.push(defaults::FIRECRACKER_VMID_DIR);
    meta_dirs.push(defaults::FIRECRACKER_OVERLAY_DIR);
    for meta_dir in meta_dirs {
        if ! Path::new(meta_dir).is_dir() {
            fs::create_dir(meta_dir).unwrap_or_else(|why| {
                panic!("Failed to create {}: {:?}", meta_dir, why.kind());
            });
            let attr = fs::metadata(meta_dir).unwrap_or_else(|why| {
                panic!(
                    "Failed to fetch {} attributes: {:?}", meta_dir, why.kind()
                );
            });
            let mut permissions = attr.permissions();
            permissions.set_mode(0o777);
            fs::set_permissions(meta_dir, permissions).unwrap_or_else(|why| {
                panic!(
                    "Failed to set {} permissions: {:?}", meta_dir, why.kind()
                );
            });
        }
    }
}

pub fn vm_running(vmid: &String, user: &String) -> bool {
    /*!
    Check if VM with specified vmid is running
    !*/
    let mut running_status = false;
    if vmid == "0" {
        return running_status
    }
    let mut running = Command::new("sudo");
    if ! user.is_empty() {
        running.arg("--user").arg(&user);
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
    let args: Vec<String> = env::args().collect();
    let mut meta_file = format!(
        "{}/{}", target_dir, program_name
    );
    for arg in &args[1..] {
        if arg.starts_with("@") {
            // The special @NAME argument is not passed to the
            // actual call and can be used to run different VM
            // instances for the same application
            meta_file = format!("{}{}", meta_file, arg);
        }
    }
    meta_file = format!("{}.{}", meta_file, extension);
    meta_file
}

pub fn gc_meta_files(
    vm_id_file: &String, user: &String, resume: bool
) -> bool {
    /*!
    Check if VM exists according to the specified
    vm_id_file. Garbage cleanup the vm_id_file
    if no longer present. Return a true value if the VM
    exists, in any other case return false.
    !*/
    let mut vmid_status = false;
    match fs::read_to_string(&vm_id_file) {
        Ok(vmid) => {
            if ! vm_running(&vmid, &user) {
                debug(&format!("Deleting {}", vm_id_file));
                match fs::remove_file(&vm_id_file) {
                    Ok(_) => { },
                    Err(error) => {
                        error!("Failed to remove VMID: {:?}", error)
                    }
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

pub fn gc(user: &String) {
    /*!
    Garbage collect VMID files for which no VM exists anymore
    !*/
    let mut vmid_file_names: Vec<String> = Vec::new();
    let mut vmid_file_count: i32 = 0;
    let paths = fs::read_dir(defaults::FIRECRACKER_VMID_DIR).unwrap();
    for path in paths {
        vmid_file_names.push(format!("{}", path.unwrap().path().display()));
        vmid_file_count = vmid_file_count + 1;
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
        gc_meta_files(&vm_id_file, &user, true);
    }
}
