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
/**
    Module implements incomming client connection and handles commands and responses 
    towards client.
 */
use std::io::{Read,Write};
use std::os::unix::net::{UnixStream, UnixListener};
use firecracker_service_communication::service_proto::{Command,Vm,Response,SOCK_NAME};
use std::collections::HashMap;

fn client_ps(vm_cont: &mut HashMap<String, Vm>)->Response{
    /*!
        Return a Response struct with list of currently running Vm's
     */
    let mut jres = Response::new();    
    jres.vm_list = Some(vm_cont.clone().into_values().collect());
    jres
}

fn client_register(vm: &Option<Vm>, vm_cont: &mut HashMap<String, Vm>)->Response{
    /*!
        Register new running Vm
    */

    let mut jres = Response::new();    
    match vm{
        Some(vm) => {
            vm_cont.insert(vm.id.clone(),vm.clone());                                    
        },
        None =>{ 
            jres.error_msg=Some("Missing id of vm to register".to_string());
            jres.ok = false;
        }   
    }
    jres
}

fn client_unregister(vm: &Option<Vm>, vm_cont: &mut HashMap<String, Vm>)->Response {
    /*!
        Unregister running Vm
    */

    let mut jres = Response::new();    
    match vm{
        Some(vm) => {
            vm_cont.remove(&vm.id);                                    
        },
        None => {
            jres.error_msg=Some("Missing id of vm to register".to_string());
            jres.ok = false;                                    
        }
    }
    jres
}

fn handle_client( client: UnixStream, vm_cont: &mut HashMap<String, Vm>){
    /*!
        respond to commands, like register or unregister new instance of mvm
        return running mvm's, run command on certain mvm etc
        protocol is simple json Command-Response 
    */
    let mut stream;
    
    match client.try_clone(){
        Ok(s) => stream = s,
        Err(_) => {
            error!("Unable to clone client stream, stream already closed");
            return
        }
    }

    let mut response = String::new();
    let mut jres;
    match stream.read_to_string(&mut response){
        Ok(_) => {
            match Command::from_json(&response){
                Ok(cmd) =>{
                    match cmd.name.as_str() {
                        "ps" => jres=client_ps(vm_cont),
                        "register" => jres=client_register(&cmd.vm, vm_cont),
                        "unregister" => jres=client_unregister(&cmd.vm, vm_cont),
                        _ => {
                            jres = Response::new();
                            jres.ok = false;
                            jres.error_msg= Some("Unknown command ".to_string());
                            error!("Unknown command {}", cmd.name)
                        }
                    }
                },
                Err(e) => {
                    error!("Wrong proto message {:}",e);
                    jres = Response::new();
                    jres.ok=false;
                    jres.error_msg = Some("Protocol error".to_string());
                }                         
            };
        },
        Err(_) => {
            error!("Reading from client stream failed");
            return
        }        
    };
    let buf;
    match jres.to_json(){
        Ok(v) =>  {
            buf = v.as_bytes();
            match stream.write_all(buf){
                Err(e) => error!("Error when sending the response: {:}",e),
                Ok(_) => debug!("Response send: {}", v)
            }
        },
        Err(e) => {
            error!("Wrong json conversion: {:}",e);
        }
    };
}

pub fn handle_incoming_connections(){
    /*!
        handle incomming connections, if connection is correct go to handle the 
        incomming stream from the socket, it is always request -> response 
    */
    let srv_socket;
    let mut vm_db: HashMap<String,Vm> = HashMap::new();
    match UnixListener::bind(SOCK_NAME){
        Ok(stream) => srv_socket = stream,
        Err(_) => {
            error!("Unable to bind to a socket {SOCK_NAME}");
            return
        }
    }
    info!("Awaiting incomming connections");

    for client in srv_socket.incoming(){
        match client{
            Ok(client) => {
                handle_client(client, &mut vm_db);
            }
            Err(e) => {
                error!("Error on incomming connection: {:}",e);
                break;
            }
        }
    }
}
