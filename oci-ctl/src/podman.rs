//
// Copyright (c) 2022 Elektrobit Automotive GmbH
//
// This file is part of oci-pilot
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
use std::process::Command;
use crate::defaults;

pub fn load(oci: &String) -> i32 {
    /*!
    Call podman load with the provided oci tar file
    !*/
    let mut status_code = 255;

    info!("Loading OCI image...");
    info!("podman load -i {}", oci);
    let status = Command::new(defaults::PODMAN_PATH)
        .arg("load")
        .arg("-i")
        .arg(oci)
        .status();

    match status {
        Ok(status) => {
            status_code = status.code().unwrap();
            if ! status.success() {
                error!("Failed, error message(s) reported");
            }
        }
        Err(status) => { error!("Process terminated by signal: {}", status) }
    }

    status_code
}

pub fn rm(container: &String){
    /*!
    Call podman image rm with force option to remove all running containers
    !*/
    info!("Removing image and all running containers...");
    info!("podman rm -f  {}", container);
    let status = Command::new(defaults::PODMAN_PATH)
        .arg("image")
        .arg("rm")
        .arg("-f")
        .arg(container)
        .status();

    match status {
        Ok(status) => {
            if ! status.success() {
                error!("Failed, error message(s) reported");
            }
        }
        Err(status) => { error!("Process terminated by signal: {}", status) }
    }
}
