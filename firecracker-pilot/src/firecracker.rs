use std::ffi::OsStr;
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
use flakes::command::{CommandError, handle_output, CommandExtTrait};
use flakes::error::{FlakeError, OperationError};
use flakes::user::{User, mkdir};
use spinoff::{Spinner, spinners, Color};
use ubyte::ByteUnit;
use std::path::Path;
use std::process::{Stdio, id};
use std::env;
use std::fs;
use crate::config::{config, RuntimeSection, EngineSection};
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

pub fn create(program_name: &String) -> Result<(String, String), FlakeError> {
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

    // setup VM ID file name
    let vm_id_file_path = get_meta_file_name(
        program_name, defaults::FIRECRACKER_VMID_DIR, "vmid"
    );

    // get flake config sections
    let RuntimeSection { runas, resume, firecracker: engine_section, .. } = config().runtime();

    // check for includes
    let tar_includes = config().tars();
    let has_includes = !tar_includes.is_empty();

    // Make sure meta dirs exists
    init_meta_dirs()?;

    // Check early return condition
    if Path::new(&vm_id_file_path).exists() && gc_meta_files(&vm_id_file_path, runas, program_name, resume)? && resume {
        // VM exists
        // report ID value and its ID file name
        let vmid =  fs::read_to_string(&vm_id_file_path)?;
        return Ok((vmid, vm_id_file_path));
    }

    // Garbage collect occasionally
    gc(runas, program_name).ok();

    // Sanity check
    if Path::new(&vm_id_file_path).exists() {
        // we are about to create a VM for which a
        // vmid file already exists.
        return Err(FlakeError::AlreadyRunning)
    }

    // Setup VM...
    let spinner = Spinner::new_with_stream(
        spinners::Line, "Launching flake...", Color::Yellow, spinoff::Streams::Stderr
    );

    match run_creation(&vm_id_file_path, program_name, engine_section, resume, runas, has_includes) {
        Ok(result) => {
            spinner.success("Launching flake");
            Ok(result)
        },
        Err(err) => {
            spinner.fail("Flake launch has failed");
            Err(err)            
        },
    }
}

fn run_creation(
    vm_id_file_path: &str,
    program_name: &String,
    engine_section: EngineSection,
    resume: bool,
    runas: User,
    has_includes: bool
) -> Result<(String, String), FlakeError> {


    // Create initial vm_id_file with process ID set to 0
    
    std::fs::File::create(vm_id_file_path)?.write_all("0".as_bytes())?;
    let result = ("0".to_owned(), vm_id_file_path.to_owned());

    // Setup root overlay if configured
    let vm_overlay_file = get_meta_file_name(
        program_name, defaults::FIRECRACKER_OVERLAY_DIR, "ext2"
    );
    if let Some(overlay_size) = engine_section.overlay_size {
        let overlay_size = overlay_size.parse::<ByteUnit>().expect("could not parse overlay size").as_u64();
        if !Path::new(&vm_overlay_file).exists() || !resume {

            let mut vm_overlay_file_fd = File::create(&vm_overlay_file)?;
            vm_overlay_file_fd.seek(SeekFrom::Start(overlay_size - 1))?; // Maybe new Error for No space left on device
            vm_overlay_file_fd.write_all(&[0])?;

            // Create filesystem
            let mut mkfs = runas.run("mkfs.ext2");
            mkfs.arg("-F")
                .arg(&vm_overlay_file);
            debug(&format!("sudo {:?}", mkfs.get_args()));
            mkfs.perform()?;
        }
    }

    // Provision VM
    let vm_image_file = engine_section.rootfs_image_path;
    
    let tmp_dir = tempdir()?;
    if let Some(tmp_dir) = tmp_dir.path().to_str() {
        let vm_mount_point = mount_vm(
            tmp_dir,
            vm_image_file,
            &vm_overlay_file,
            User::ROOT
        )?;
        if has_includes {
            debug("Syncing includes...");
            sync_includes(&vm_mount_point, User::ROOT)?;
        }
        umount_vm(tmp_dir, User::ROOT)?;
    }

    Ok(result)

}

pub fn start(
    program_name: &String, (vm_id, vm_id_file): (String, String)
) -> Result<(), FlakeError> {
    /*!
    Start VM with the given VM ID

    firecracker-pilot exits with the return code from firecracker
    after this function
    !*/
    let RuntimeSection { runas, resume, .. } = config().runtime();

    let mut is_blocking: bool = true;


    if vm_running(&vm_id, runas)? {
        // 1. Execute app in running VM
        execute_command_at_instance(
            program_name, runas, get_exec_port()
        )?;
    } else {
        let firecracker_config = NamedTempFile::new()?;
        create_firecracker_config(
            program_name, &firecracker_config
        )?;
        if resume {
            // 2. Startup resume type VM and execute app
            is_blocking = false;
            call_instance(
                &firecracker_config, &vm_id_file, runas, is_blocking
            )?;
            execute_command_at_instance(
                program_name, runas, get_exec_port()
            )?;
        } else {
            // 3. Startup VM and execute app
            call_instance(
                &firecracker_config, &vm_id_file, runas, is_blocking
            )?;
        }
        
    }

    Ok(())
}

pub fn call_instance(
    config_file: &NamedTempFile, vm_id_file: &String,
    user: User, is_blocking: bool
) -> Result<(), FlakeError> {
    /*!
    Run firecracker with specified configuration
    !*/
    let mut firecracker = user.run("firecracker");
    if ! is_debug() {
        firecracker.stderr(Stdio::null());
    }
    if ! is_debug() && ! is_blocking {
        firecracker
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
    }
    firecracker
        .arg("--no-api")
        .arg("--id")
        .arg(id().to_string())
        .arg("--config-file")
        .arg(config_file.path());
    debug(&format!("sudo {:?}", firecracker.get_args()));

    let child = firecracker.spawn()?;
    let pid = child.id();
    debug(&format!("PID {}", pid));

    File::create(vm_id_file)?.write_all(pid.to_string().as_bytes())?;

    if is_blocking {
        handle_output(child.wait_with_output(), firecracker.get_args())?;
    }
    Ok(())
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

pub fn check_connected(program_name: &String, user: User) -> Result<(), FlakeError> {
    /*!
    Check if instance connection is OK
    !*/
    let mut retry_count = 0;
    let vsock_uds_path = format!(
        "/run/sci_cmd_{}.sock", get_meta_name(program_name)
    );
    loop {
        if retry_count == defaults::RETRIES {
            debug("Max retries for VM connection check exceeded");
            return Err(FlakeError::OperationError(OperationError::MaxTriesExceeded))
        }
        let mut vm_command = user.run("bash");
        vm_command.arg("-c")
            .arg(&format!(
                "echo -e 'CONNECT {}'|{} UNIX-CONNECT:{} -",
                defaults::VM_PORT,
                defaults::SOCAT,
                vsock_uds_path
            ));
        debug(&format!("sudo {:?}", vm_command.get_args()));
        let output = vm_command.output()?;
        if String::from_utf8_lossy(&output.stdout).starts_with("OK") {
            return Ok(())
        }
        
        // VM not ready for connections
        let some_time = time::Duration::from_millis(
            defaults::VM_WAIT_TIMEOUT_MSEC
        );
        thread::sleep(some_time);
        retry_count += 1
    }
}

pub fn send_command_to_instance(
    program_name: &String, user: User, exec_port: u32
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
        let mut vm_command = user.run("bash");
        vm_command.arg("-c")
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
    program_name: &String, user: User, exec_port: u32
) -> Result<(), FlakeError> {
    /*!
    Send command to a vsoc connected to a running instance
    !*/
    let mut retry_count = 0;
    let vsock_uds_path = format!(
        "/run/sci_cmd_{}.sock", get_meta_name(program_name)
    );

    // wait for UDS socket to appear
    loop {
        if retry_count == defaults::RETRIES {
            debug("Max retries for UDS socket lookup exceeded");
            Err(OperationError::MaxTriesExceeded)?
        }
        if Path::new(&vsock_uds_path).exists() {
            break
        }
        let some_time = time::Duration::from_millis(100);
        thread::sleep(some_time);
        retry_count += 1
    }

    // make sure instance can be contacted
    check_connected(program_name, user)?;

    // spawn the listener and wait for sci to run the command
    let mut vm_exec = user.run(defaults::SOCAT);
    vm_exec.arg("-t")
        .arg("0")
        .arg("-")
        .arg(
            &format!("UNIX-LISTEN:{}_{}",
            vsock_uds_path, exec_port
        ));
    debug(&format!("sudo {:?}", vm_exec.get_args()));
    let child = vm_exec.spawn()?;
    send_command_to_instance(
        program_name, user, exec_port
    );
    handle_output(child.wait_with_output(), vm_exec.get_args())?;
    Ok(())
}

pub fn create_firecracker_config(
    program_name: &String,
    config_file: &NamedTempFile
) -> Result<(), FlakeError> {
    /*!
    Create json config to call firecracker
    !*/
    let template = File::open(defaults::FIRECRACKER_TEMPLATE)?;
    let mut firecracker_config: FireCrackerConfig = serde_json::from_reader(template)?;
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

    debug(&serde_json::to_string(&firecracker_config)?);
    serde_json::to_writer(
        config_file, &firecracker_config
    )?;

    Ok(())
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

pub fn init_meta_dirs() -> Result<(), CommandError>{
    [defaults::FIRECRACKER_OVERLAY_DIR, defaults::FIRECRACKER_VMID_DIR].iter()
        .filter(|path| !Path::new(path).is_dir())
        .try_for_each(|path| mkdir(path, "777", User::ROOT))
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

pub fn vm_running(vmid: &String, user: User) -> Result<bool, FlakeError> {
    /*!
    Check if VM with specified vmid is running
    !*/
    debug(&format!("vm id is {vmid}"));
    if vmid == "0" {
        return Ok(false)
    }
    let mut running = user.run("kill");
    running.arg("-0").arg(vmid);
    debug(&format!("{:?}", running.get_args()));
    
    let output = running.output()?;
    Ok(output.status.success())
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
    vm_id_file: &String, user: User, program_name: &String, resume: bool
) -> Result<bool, FlakeError> {
    /*!
    Check if VM exists according to the specified
    vm_id_file. Garbage cleanup the vm_id_file and the vsock socket
    if no longer present. Return a true value if the VM
    exists, in any other case return false.
    !*/
    let mut vmid_status = false;
    match fs::read_to_string(vm_id_file) {
        Ok(vmid) => {
            if ! vm_running(&vmid, user)? {
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
                    "{}/{}", 
                    defaults::FIRECRACKER_OVERLAY_DIR,
                    Path::new(&vm_id_file)
                        .file_name()
                        .and_then(OsStr::to_str)
                        .map(|x| x.replace(".vmid", ".ext2"))
                        .unwrap()
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
    Ok(vmid_status)
}

pub fn gc(user: User, program_name: &String) -> Result<(), FlakeError> {
    /*!
    Garbage collect VMID files for which no VM exists anymore
    !*/
    let vmid_file_names: Vec<_> = fs::read_dir(defaults::FIRECRACKER_VMID_DIR)?
        .filter_map(|entry| entry.ok())
        .filter_map(|x| x.path()
            .to_str()
            .map(ToOwned::to_owned))
        .collect();

    if vmid_file_names.len() <= defaults::GC_THRESHOLD {
        return Ok(())
    }
    for vm_id_file in vmid_file_names {
        // collective garbage collect but do not delete overlay
        // images as they might be re-used for resume type instances.
        // The cleanup of overlay images from resume type instances
        // must be done by an explicit user action to avoid deleting
        // user data in overlay images eventually preserved for later.
        gc_meta_files(&vm_id_file, user, program_name, true).ok();
    }
    Ok(())
}

pub fn delete_file(filename: &String, user: User) -> bool {
    /*!
    Delete file via sudo
    !*/
    let mut call = user.run("rm");
    call.arg("-f").arg(filename);
    match call.status() {
        Ok(_) => { },
        Err(error) => {
            error!("Failed to rm: {}: {:?}", filename, error);
            return false
        }
    }
    true
}

// Todo: unifiy with podman
pub fn sync_includes(
    target: &String, user: User
) -> Result<(), FlakeError> {
    /*!
    Sync custom include data to target path
    !*/
    let tar_includes = &config().tars();
    
    for tar in tar_includes {
        debug(&format!("Adding tar include: [{}]", tar));
        let mut call = user.run("tar");
        call.arg("-C").arg(target)
            .arg("-xf").arg(tar);
        debug(&format!("{:?}", call.get_args()));
        let output = call.perform()?;
        debug(&String::from_utf8_lossy(&output.stdout));
        debug(&String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
}

pub fn mount_vm(
    sub_dir: &str, rootfs_image_path: &str,
    overlay_path: &str, user: User
) -> Result<String, FlakeError> {
    /*!
    Mount VM with overlay below given sub_dir
    !*/
    // 1. create overlay image mount structure
    [
        defaults::IMAGE_ROOT,
        defaults::IMAGE_OVERLAY
    ].iter()
        .map(|p| format!("{}/{}", sub_dir, p))
        .filter(|path| !Path::new(path).exists())
        .try_for_each(fs::create_dir_all)?;

    // 2. mount VM image
    let image_mount_point = format!(
        "{}/{}", sub_dir, defaults::IMAGE_ROOT
    );
    let mut mount_image = user.run("mount");
    mount_image.arg(rootfs_image_path)
        .arg(&image_mount_point);
    debug(&format!("{:?}", mount_image.get_args()));
    mount_image.perform()?;
    // 3. mount Overlay image
    let overlay_mount_point = format!(
        "{}/{}", sub_dir, defaults::IMAGE_OVERLAY
    );
    let mut mount_overlay = user.run("mount");
    mount_overlay.arg(overlay_path)
        .arg(&overlay_mount_point);
    debug(&format!("{:?}", mount_overlay.get_args()));
    mount_overlay.perform()?;
    // 4. mount as overlay
    [
        defaults::OVERLAY_ROOT,
        defaults::OVERLAY_UPPER,
        defaults::OVERLAY_WORK
    ].iter()
        .map(|p| format!("{}/{}", sub_dir, p))
        .filter(|path| !Path::new(path).exists())
        .try_for_each(|path| mkdir(&path, "755", User::ROOT))?;
    
    let root_mount_point = format!("{}/{}", sub_dir, defaults::OVERLAY_ROOT);
    let mut mount_overlay = user.run("mount");
    mount_overlay.arg("-t")
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
    mount_overlay.perform()?;
    Ok(root_mount_point)
}

#[allow(clippy::needless_collect)] // defer shortcutting
pub fn umount_vm(sub_dir: &str, user: User) -> Result<(), CommandError> {
    /*!
    Umount VM image
    !*/
    let x: Vec<_> = [
        defaults::OVERLAY_ROOT,
        defaults::IMAGE_OVERLAY,
        defaults::IMAGE_ROOT,
    ].iter().map(|mount_point| {
        let mut umount = user.run("umount");
        umount.stderr(Stdio::null());
        umount.stdout(Stdio::null());
        umount.arg(format!("{}/{}", &sub_dir, &mount_point));
        debug(&format!("{:?}", umount.get_args()));
        umount.perform().map(|_| ())
    }).collect();

    x.into_iter().collect()
}
