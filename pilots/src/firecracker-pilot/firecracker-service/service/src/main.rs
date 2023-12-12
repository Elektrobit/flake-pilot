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
extern crate daemonize;

use std::fs::File;
use daemonize::Daemonize;

/**
    Module implements incomming client connection and handles commands and responses 
    towards client.
 */
mod app;
use crate::app::handle_incoming_connections;

/**
   Module defines default values like file names for output etc.
 */
mod defaults;
use crate::defaults::*;

#[macro_use]
extern crate log;

use env_logger::Env;


fn main() {
    /*! 
        firecracker-service is a service meant to run in background, to provide unix-domain socket
        that will be used to communicate with it. 
    */    
    let stdout = File::create(FC_SERVICE_OUT).unwrap();
    let stderr = File::create(FC_SERVICE_ERR).unwrap();
    
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
