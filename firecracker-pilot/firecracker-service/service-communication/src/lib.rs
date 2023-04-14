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
pub mod service_proto{
    /*! 
        Module implements structures, serialization and access point names for communication between 
        firecracker-service and firecracker-pilot
    */
    use serde::{Serialize,Deserialize};
    use serde_json::Result;

    /** 
        SOCK_NAME is service socket name which server firecracker-service API 
    */
    pub const SOCK_NAME: &str ="/run/firecracker-service.socket";
    

    /** 
        Vm represents virtual machine which is currently running 
    */
    #[derive(Clone,Debug,Serialize,Deserialize)]
    pub struct Vm{
        pub id: String,
        pub cmd: Vec< String >
    }

    /** 
        Command implements command send to the service 
    */
    #[derive(Clone,Debug,Serialize,Deserialize)]
    pub struct Command{
        pub name: String,
        pub vm: Option< Vm >
    }

    /** 
        Response implements return result from service to the client 
    */    
    #[derive(Clone,Debug,Serialize,Deserialize)]
    pub struct Response{    
        pub ok: bool,
        pub vm_list: Option< Vec< Vm > >,
        pub error_msg: Option< String >
    }

    impl Command{
        pub fn from_json(text: &str) -> Result< Command > {
            /*! 
                from_json deserializes Command into Result
            */
            serde_json::from_str(text)
        }

        pub fn to_json(&self) -> Result< String > {
            /*! 
                to_json serializes Command into Result
            */
            serde_json::to_string(&self)
        }
    }

    impl Response{
        pub fn new() -> Response {
            /*! 
                creates new Result object and initialises the fields with default values
            */
            Response{ ok: true, 
                vm_list: None,
                error_msg:None}
        }

        pub fn form_json(text: &str) -> Result< Response > {
            /*! 
                from_json deserializes Response into Result
            */
            serde_json::from_str(text)
        }
        
        pub fn to_json(&self) -> Result< String > {
            /*! 
                to_json serializes Response into Result
            */
            serde_json::to_string(&self)
        }
    }
}
