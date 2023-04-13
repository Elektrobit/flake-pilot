extern crate daemonize;

use std::fs::File;
use daemonize::Daemonize;

/**
    Module implements incomming client connection and handles commands and responses 
    towards client.
 */
mod app;
use crate::app::handle_incoming_connections;


#[macro_use]
extern crate log;

use env_logger::Env;


fn main() {
    /*! 
        firecracker-service is a service meant to run in background, to provide unix-domain socket
        that will be used to communicate with it. 
    */    
    let stdout = File::create("/tmp/firecracker-service.out").unwrap();
    let stderr = File::create("/tmp/firecracker-service.err").unwrap();
    
    setup_logger();

    let daemonize = Daemonize::new()
        .pid_file("/var/run/firecracker-service.pid") 
        .chown_pid_file(true)    
        .working_directory("/tmp")
        .user("root")
        .group("root") 
        .umask(0o777)    
        .stdout(stdout) 
        .stderr(stderr);        

    match daemonize.start() {
        Ok(_) => {
            info!("Started daemon ...");
            handle_incoming_connections();            
        },
        Err(e) => error!("Error, {}", e),
    }
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
