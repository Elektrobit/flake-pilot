use std::io::Read;
use std::thread;
use std::os::unix::net::{UnixStream, UnixListener};

const SOCK_NAME: &str ="/var/run/firecracker-service.socket";

fn handle_client( client: UnixStream){
    /*!
        respond to commands, like register or unregister new instance of mvm
        return running mvm's, run command on certain mvm etc
        protocol can be simple json, TBD
    !*/
    let mut stream;
    
    match client.try_clone(){
        Ok(s) => stream = s,
        Err(_) => {
            error!("Unable to clone client stream, stream already closed");
            return
        }
    }

    let mut response = String::new();
    match stream.read_to_string(&mut response){
        Ok(_) => {
            // TBD parse incoming request and handle it
            info!("{response}");
        },
        Err(_) => {
            error!("Reading from client stream failed");
            return
        }
    }
    
}

pub fn handle_incoming_connections(){
    /*!
        handle incomming connections, if connection is correct go to handle the 
        incomming stream from the socket, it is always request -> response 
        !*/
    let srv_socket ;
    
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
                thread::spawn(|| handle_client(client));
            }
            Err(e) => {
                error!("Error on incomming connection: {:}",e);
                break;
            }
        }
    }
}
