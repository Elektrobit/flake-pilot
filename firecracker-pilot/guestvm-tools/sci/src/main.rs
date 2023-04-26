use std::env;

use std::process::{Command,exit};
use std::os::unix::process::CommandExt;
use std::io::Error;
use std::fs;

use sys_mount::Mount;

#[macro_use]
extern crate log;

use env_logger::Env;

/**
    SCE is an executable for Simple Command Executor
 */
const SCE: &str = "/sbin/sce";

/**
    SWITCH_ROOT is a switch_root tool path
 */
const SWITCH_ROOT: &str = "/usr/sbin/switch_root";

/** 
    OVERLAY_DIR is a destination overlay dir 
 */
const OVERLAY_DIR: &str = "/overlay/rootfs";

fn main() {
     /*! Simple Command Init is a tool which is responsible to read variable *overlay_root* and 
        mount proper overlay devices if set. When all mount's are finished it has to start the 
        sce (Simple Command Executor) as main process switching the root to the overlay fs.
        Note: all environment variables are passed along by the kernel.  
    */  
    setup_logger();

    let overlay_root = match env::var("overlay_root").ok() {
        Some( v ) => v,
        None => "normal".to_string()
    };
    
    // mount proc fs and sysfs
    let m_result = Mount::builder()
        .fstype("proc")
        .mount("proc","/proc");
    match m_result{
        Ok(_) => info!("Mounted proc succesfully"),
        Err(e) =>{
            error!("Failed to mount proc: {:}",e);
            exit(1);
        }
    }
    let m_result = Mount::builder()
        .fstype("sysfs")
        .mount("sysfs","/sys");
    match m_result{
        Ok(_) => info!("Mounted sysfs succesfully"),
        Err(e) =>{
            error!("Failed to mount sysfs: {:}",e);
            exit(1);
        }
    }

    let m_result = Mount::builder()
        .fstype("tmpfs")
        .mount("tmpfs","/run");
    match m_result{
        Ok(_) => info!("Mounted run succesfully"),
        Err(e) =>{
            error!("Failed to mount run: {:}",e);
            exit(1);
        }
    }

    match overlay_root.as_str() {
        "normal" => {
            // overlay not set, start sce without any additional mounting
            info!("Starting sce without mounting overlayfs");          
            execute(SCE, &[]);
        },
        ovr => {          
            // overlay device set, mount the device and prepare the folder structure
            info!("Mounting overlayfs");
            let m_result = Mount::builder()
                .fstype("ext4")
                .mount(format!("/dev/{}", ovr),"/overlay");
            match m_result{
                Ok(_) => info!("Mounted overlay succesfully"),
                Err(e) =>{
                    error!("Failed to mount overlay: {:}",e);
                    exit(1);
                }
            }
        
            info!("Create directory structure");
            match fs::create_dir_all("/overlay/rootfs"){
                Ok(_) => {},
                Err(e) => {
                    error!("Error occures when creating directory: {:}",e);
                    exit(1);
                }
            }
        
            match fs::create_dir_all("/overlay/rootfs_upper"){
                Ok(_) => {},
                Err(e) => {
                    error!("Error occures when creating directory: {:}",e);
                    exit(1);
                }
            }
        
            match fs::create_dir_all("/overlay/rootfs_work"){
                Ok(_) => {},
                Err(e) => {
                    error!("Error occures when creating directory: {:}",e);
                    exit(1);
                }
            }

            info!("Mount the working and lower directories");
            let m_result = Mount::builder()
                .fstype("overlay")
                .data("lowerdir=/,upperdir=/overlay/rootfs_upper,workdir=/overlay/rootfs_work")                
                .mount("overlay", "/overlay/rootfs");
            match m_result{
                Ok(_) => info!("Mounted overlay succesfully"),
                Err(e) =>{
                    error!("Failed to mount overlay: {:}",e);
                    exit(1);
                }
            }
        
            info!("Execute sce tool");
            execute(SWITCH_ROOT, &[OVERLAY_DIR, SCE]);
        }
    }   
}

fn execute(exe: &str, args: &[&str]) -> Error {
    /*!
        Execute in place command with arguments, equivalent to execv in linux
     */
    info!("Executed process : {} ", exe);
    Command::new(exe).args(args).exec()
}

fn setup_logger() {
    /*!
        Set up the logger internally
    */
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "trace")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}
