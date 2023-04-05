pub mod app{
    use std::io::Read;
    use std::thread;
    use std::os::unix::net::{UnixStream, UnixListener};
    
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
                // TBD handle error
                return
            }
        }

        let mut response = String::new();
        match stream.read_to_string(&mut response){
            Ok(_) => {
                // TBD parse incoming request and handle it
                println!("{response}");
            },
            Err(_) => {
                // TBD handle error 
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
        match UnixListener::bind("/var/run/firecracker-service.socket"){
            Ok(stream) => srv_socket = stream,
            Err(_) => {
                // TBD handle error
                return
            }
        }
        println!("Awaiting incomming connections");
        
        for client in srv_socket.incoming(){
            match client{
                Ok(client) => {
                    thread::spawn(|| handle_client(client));
                }
                Err(_) => {
                    // TBD handle error
                    break;
                }
            }
        }
    }
}