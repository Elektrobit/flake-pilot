use std::process::Command;
use std::process::ExitStatus;
use std::process::exit;
use std::path::Path;
use std::os::unix::fs::symlink;

extern crate clap;
use clap::{Arg, App,SubCommand};

const VERSION:&str = "0.1";


fn oci_register(tar_name: &str){
    let mut app = Command::new("podman");

    app.arg("image")
        .arg("load")
        .arg("-i")
        .arg(tar_name);

    let status = app.status()
        .expect(&format!("Failed to execute with error"));

    match status.code() {
        Some(code) => exit(code),
        None => println!("Process terminated by signal")
    }
}

fn oci_deregister(cont_name: &str){
    let mut app = Command::new("podman");
    
    app.arg("image")
        .arg("rm")
        .arg("--force")
        .arg(cont_name);

    let status = app.status()
        .expect(&format!("Failed to execute with error"));

    match status.code() {
        Some(code) => exit(code),
        None=> println!("Process terminated by signal")
    }
} 

fn oci_expose(app: &str, cont_name: &str, cont_folder: &str){
    let result=symlink(format!("{}{}",cont_folder,cont_name),
       format!("{}",app) );
    
}

fn main() {

    let matches = App::new("oci-register")
                .about("Registers the container in the registry and applications for use within registry")
                .version(VERSION)
                .author("")
                .subcommand(SubCommand::with_name("register")
                    .about("Register container in the registry, run as oci-register register tar_file")
                    .arg(Arg::with_name("TAR")
                        .help("Tar file of the oci-container")
                        .required(true)
                        .index(1)))
                .subcommand(SubCommand::with_name("expose")
                    .about("Exposes the application registering it for certain container")
                    .arg(Arg::with_name("app")
                        .short("a")
                        .long("app")
                        .value_name("P  ATH")
                        .help("Path to the application binary for example /opt/usr/bin/python3")
                        .required(true))
                    .arg(Arg::with_name("container_folder")
                        .short("f")
                        .long("cont_folder")
                        .value_name("C_FOLDER")
                        .default_value("/usr/bin/")
                        .help("Path to the container folder")
                        .required(false))
                    .arg(Arg::with_name("container")
                        .short("c")
                        .long("cont")
                        .value_name("CONT_NAME")
                        .help("Container name within which one the registered application shall run")
                        .required(true)))
                .subcommand(SubCommand::with_name("deregister")
                    .about("De-register container in the registry, run as oci-register deregister container_name")
                    .arg(Arg::with_name("CONTAINER_NAME")
                            .help("Sets the container name ")
                            .required(true)
                            .index(1))).get_matches();
    
    if let Some(matches) = matches.subcommand_matches("register"){
        let tar_file  = matches.value_of("TAR").unwrap();
        oci_register(tar_file);

    } else if let Some(matches) = matches.subcommand_matches("expose"){
        let app_path = matches.value_of("PATH").unwrap();
        let cont_name = matches.value_of("CONT_NAME").unwrap();
        let cont_folder = matches.value_of("C_FOLDER").unwrap();
        oci_expose(app_path, cont_name, cont_folder);

    } else if let Some(matches) = matches.subcommand_matches("deregister"){
        let cont_name  = matches.value_of("CONTAINER_NAME").unwrap();
        oci_deregister(cont_name);
    }    
    
}
