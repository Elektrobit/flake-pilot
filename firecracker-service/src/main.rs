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
    let stdout = File::create("/tmp/daemon.out").unwrap();
    let stderr = File::create("/tmp/daemon.err").unwrap();
    

    let daemonize = Daemonize::new()
        .pid_file("/var/run/firecracker-service.pid") 
        .chown_pid_file(true)    
        .working_directory("/tmp")
        .user("root")
        .group("root") 
        .umask(0o777)    
        .stdout(stdout) 
        .stderr(stderr) 
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => handle_incoming_connections(),
        Err(e) => println!("Error, {}", e),
    }



}
