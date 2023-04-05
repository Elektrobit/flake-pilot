extern crate daemonize;

use std::fs::File;
use daemonize::Daemonize;

mod app;
use crate::app::app::handle_incoming_connections;


fn main() {
    /*! 
        firecracker-service is a service meant to run in background, to provide unix-domain socket
        that will be used to communicate with it. 
    !*/
    let stdout = File::create("/tmp/firecracker-service.out").unwrap();
    let stderr = File::create("/tmp/firecracker-service.err").unwrap();
    

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
            println!("Started daemon ...");
            handle_incoming_connections();            
        },
        Err(e) => println!("Error, {}", e),
    }



}
