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
#[macro_use]
extern crate log;
extern crate shell_words;

pub mod defaults;

use std::env;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;
use std::os::unix::process::{CommandExt, ExitStatusExt};
use system_shutdown::force_reboot;
use std::fs;
use sys_mount::Mount;
use env_logger::Env;
use std::{thread, time};
use vsock::{VsockListener};
use std::io::{Read, Write};
use std::process::{Stdio, Output, ExitStatus};
use std::net::Shutdown;

use crate::defaults::debug;



fn main() {
    /*!
    Simple Command Init (sci) is a tool which executes the provided
    command in the run=... cmdline variable after preparation of an
    execution environment for the purpose to run a command inside
    of a firecracker instance.

    if provided via the overlay_root=/dev/block_device kernel boot
    parameter, sci also prepares the root filesystem as an overlay
    using the given block device for writing.
    !*/
    setup_logger();

    let mut args: Vec<String> = vec![];
    let mut call: Command;
    let mut do_exec = false;
    let mut ok = true;

    // print user space env
    for (key, value) in env::vars() {
        debug(&format!("{}: {}", key, value));
    }

    // parse commandline from run environment variable
    match env::var("run").ok() {
        Some(call_cmd) => {
            match shell_words::split(&call_cmd) {
                Ok(call_params) => {
                    args = call_params
                },
                Err(error) => {
                    debug(&format!("Failed to parse {}: {}", call_cmd, error));
                    ok = false
                }
            }
        },
        None => {
            debug(&format!("No run=... cmdline parameter in env"));
            do_reboot(ok)
        }
    }

    // sanity check on command to call
    if args[0].is_empty() {
        debug("No command to execute specified");
    }

    // check if given command requires process replacement
    if args[0] == "/usr/lib/systemd/systemd" {
        do_exec = true;
    }

    // mount /proc, /sys and /run, skip if already mounted
    mount_basic_fs();

    // mount overlay if requested
    match env::var("overlay_root").ok() {
        Some(overlay) => {
            // overlay device is specified, mount the device and
            // prepare the folder structure
            let mut modprobe = Command::new(defaults::PROBE_MODULE);
            modprobe.arg("overlay");
            debug(&format!(
                "CALL: {} -> {:?}", defaults::PROBE_MODULE, modprobe.get_args()
            ));
            match modprobe.status() {
                Ok(_) => { },
                Err(error) => {
                    debug(&format!("Loading overlay module failed: {}", error));
                }
            }
            debug(&format!("Mounting overlayfs RW({})", overlay.as_str()));
            match Mount::builder()
                .fstype("ext2").mount(overlay.as_str(), "/overlayroot")
            {
                Ok(_) => {
                    debug(&format!("Mounted {:?} on /overlayroot", overlay));
                    ok = true
                },
                Err(error) => {
                    debug(&format!("Failed to mount overlayroot: {}", error));
                    ok = false
                }
            }
            if ok {
                let overlay_dirs = vec![
                    defaults::OVERLAY_ROOT,
                    defaults::OVERLAY_UPPER,
                    defaults::OVERLAY_WORK
                ];
                for overlay_dir in overlay_dirs.iter() {
                    match fs::create_dir_all(overlay_dir) {
                        Ok(_) => { ok = true },
                        Err(error) => {
                            debug(&format!(
                                "Error creating directory {}: {}",
                                defaults::OVERLAY_ROOT, error
                            ));
                            ok = false;
                            break;
                        }
                    }
                }
            }
            if ok {
                match Mount::builder()
                    .fstype("overlay")
                    .data(
                        &format!("lowerdir=/,upperdir={},workdir={}",
                            defaults::OVERLAY_UPPER, defaults::OVERLAY_WORK
                        )
                    )
                    .mount("overlay", "/overlayroot/rootfs")
                {
                    Ok(_) => {
                        debug(&format!(
                            "Mounted overlay on {}", defaults::OVERLAY_ROOT
                        ));
                        ok = true;
                    },
                    Err(error) => {
                        debug(&format!(
                            "Failed to mount overlayroot: {}", error
                        ));
                        ok = false;
                    }
                }
            }
            // Call specified command through switch root into the overlay
            if ok {
                move_mounts(defaults::OVERLAY_ROOT);
                let root = Path::new(defaults::OVERLAY_ROOT);
                match env::set_current_dir(&root) {
                    Ok(_) => {
                        debug(&format!(
                            "Changed working directory to {}", root.display()
                        ));
                        ok = true;
                    },
                    Err(error) => {
                        debug(&format!(
                            "Failed to change working directory: {}", error
                        ));
                        ok = false;
                    }
                }
            }
            if do_exec {
                call = Command::new(defaults::SWITCH_ROOT);
                call.arg(".").arg(&args[0]);
            } else {
                call = Command::new(&args[0]);
                if ok {
                    let mut pivot = Command::new(defaults::PIVOT_ROOT);
                    pivot.arg(".").arg("mnt");
                    debug(&format!(
                        "CALL: {} -> {:?}",
                        defaults::PIVOT_ROOT, pivot.get_args()
                    ));
                    match pivot.status() {
                        Ok(_) => {
                            debug(&format!(
                                "{} is now the new root", defaults::OVERLAY_ROOT
                            ));
                            ok = true;
                        },
                        Err(error) => {
                            debug(&format!("Failed to pivot_root: {}", error));
                            ok = false;
                        }
                    }
                    mount_basic_fs();
                    setup_resolver_link();
                }
            }
        },
        None => {
            // Call command in current environment
            call = Command::new(&args[0]);
        }
    };

    // Setup command call parameters
    for arg in &args[1..] {
        call.arg(arg);
    }

    // Execute command
    // TODO: At this point we want to redirect the command output
    // maybe to a vsoc. At the moment this will output to whatever
    // /dev/console is but is intermixed with any other message
    // on that console, e.g the kernel log. This is also the reason
    // why no action on the status is implemented so far.
    //
    // Also see: https://github.com/Elektrobit/flake-pilot/issues/75
    //
    // Stay tuned...

    // are we in resume state
    match env::var("sci_resume").ok() {
        Some(_) => {
            // check if vhost is loaded
            let mut modprobe = Command::new(defaults::PROBE_MODULE);
            modprobe.arg("vhost");
            debug(&format!(
                "CALL: {} -> {:?}", defaults::PROBE_MODULE, modprobe.get_args()
            ));
            match modprobe.status() {
                Ok(_) => { },
                Err(error) => {
                    debug(&format!("Loading vhost module failed: {}", error));
                }
            }
            // start vsock listener, wait for command, execute it and send back the stdout 
            // and stderr
            debug(&format!("Binding socket "));
            match VsockListener::bind_with_cid_port(3, defaults::VM_PORT){
                Ok(listener)=>{
                    // main loop
                    let mut cnt=1;
                    loop {
                        cnt=cnt+1;
                        match listener.accept(){
                            Ok((mut stream, addr)) =>{
                                debug(&format!("Accepted incoming connection from : {}:{}", addr.cid(), addr.port()));
                                args = vec!();
                                let mut buf: String = "".to_string();
                                let mut bytes = [0;4096];
                                match stream.read(&mut bytes){
                                    Ok(_)=>{
                                        buf=String::from_utf8(bytes.to_vec()).unwrap();
                                        debug(&format!("Read from string: {}",&buf));
                                    },
                                    Err(e)=>{
                                        debug(&format!("Failed to read data {}", e));
                                    }
                                };
                                // call a command and keep control
                                match shell_words::split(&buf) {
                                    Ok(call_params) => {
                                        args = call_params;
                                    },
                                    Err(error) => {
                                        debug(&format!("Failed to parse {}: {}", &buf, error));
                                    }
                                }
                                call = Command::new(&args[0]);
                                for arg in &args[1..args.len()-1] {
                                    call.arg(arg);
                                }

                                if args[0].eq(defaults::VM_QUIT) {
                                    debug(&format!("Quit command called"));
                                    break;
                                }

                                debug(&format!("CALL: {} -> {:?}", &args[0], call.get_args()));
                                call.stdout(Stdio::piped())
                                    .stderr(Stdio::piped());
                                match call.spawn() {
                                    Ok(child) => { 
                                        let output = match child.wait_with_output(){
                                            Ok( out) => out,
                                            Err(e) => {
                                                stream.write_all(&format!("Failed to run command {}: {}", buf, e).as_bytes()).unwrap();
                                                Output{status:ExitStatus::from_raw(-1) , stdout: Vec::new(), stderr: Vec::new()}
                                            }
                                        };
                                        stream.write_all(&output.stderr).unwrap();
                                        stream.write_all(&output.stdout).unwrap();
                                        stream.shutdown(Shutdown::Both).unwrap();
                                    },
                                    Err(e) => {
                                        stream.write_all(&format!("Failed to run command {}: {}", buf, e).as_bytes()).unwrap();
                                        stream.shutdown(Shutdown::Both).unwrap();
                                     }
                                }

                            },
                            Err(e) =>{
                                debug(&format!("Failed to accept incoming connection: {}", e));
                            }
                        }
                    }
                },
                Err(e)=>{
                    debug(&format!("Failed to bind vsock: CID: 3 {}", e));
                }
            }


        },
        None => {
            // run regular command and close vm
            if ok {
                if do_exec {
                    // replace ourselves
                    debug(&format!("EXEC: {} -> {:?}", &args[0], call.get_args()));
                    call.exec();
                } else {
                    // call a command and keep control
                    debug(&format!("CALL: {} -> {:?}", &args[0], call.get_args()));
                    match call.status() {
                        Ok(_) => { },
                        Err(_) => { }
                    }
                }
            }
        }

    }
    
    // Close firecracker session
    do_reboot(ok)
}

fn do_reboot(ok: bool) {
    debug("Rebooting...");
    if ! ok {
        // give potential error messages some time to settle
        let some_time = time::Duration::from_millis(10);
        thread::sleep(some_time);
    }
    match force_reboot() {
        Ok(_) => { },
        Err(error) => {
            panic!("Failed to reboot: {}", error)
        }
    }
}

fn setup_resolver_link() {
    if Path::new(defaults::SYSTEMD_NETWORK_RESOLV_CONF).exists() {
        match symlink(
            defaults::SYSTEMD_NETWORK_RESOLV_CONF, "/etc/resolv.conf"
        ) {
            Ok(_) => { },
            Err(error) => {
                debug(&format!("Error creating symlink \"{} -> {}\": {:?}",
                    "/etc/resolv.conf",
                    defaults::SYSTEMD_NETWORK_RESOLV_CONF,
                    error
                ));
            }
        }
    }
}

fn move_mounts(new_root: &str) {
    /*!
    Move filesystems from current root to new_root
    !*/
    // /run
    let mut call = Command::new("mount");
    call.arg("--bind").arg("/run").arg(&format!("{}/run", new_root));
    debug(&format!("EXEC: mount -> {:?}", call.get_args()));
    match call.status() {
        Ok(_) => debug("Bind mounted /run"),
        Err(error) => {
            debug(&format!("Failed to bind mount /run: {}", error));
            match Mount::builder()
                .fstype("tmpfs").mount("tmpfs", &format!("{}/run", new_root))
            {
                Ok(_) => debug("Mounted tmpfs on /run"),
                Err(error) => {
                    debug(&format!("Failed to mount /run: {}", error));
                }
            }
        }
    }
}

fn mount_basic_fs() {
    /*!
    Mount standard filesystems
    !*/
    match Mount::builder().fstype("proc").mount("proc", "/proc") {
        Ok(_) => debug("Mounted proc on /proc"),
        Err(error) => {
            debug(&format!("Failed to mount /proc [skipped]: {}", error));
        }
    }
    match Mount::builder().fstype("sysfs").mount("sysfs", "/sys") {
        Ok(_) => debug("Mounted sysfs on /sys"),
        Err(error) => {
            debug(&format!("Failed to mount /sys: {}", error));
        }
    }
    match Mount::builder().fstype("devtmpfs").mount("devtmpfs", "/dev") {
        Ok(_) => debug("Mounted devtmpfs on /dev"),
        Err(error) => {
            debug(&format!("Failed to mount /dev: {}", error));
        }
    }
}

fn setup_logger() {
    /*!
    Set up the logger internally
    !*/
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
