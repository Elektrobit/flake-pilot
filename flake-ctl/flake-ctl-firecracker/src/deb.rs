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
use std::process::Command;
use std::path::Path;
use std::fs;
use log::{error, info, warn};

use crate::defaults;

pub fn ocideb(
    oci: &String, repo: &String, apps: &Vec<String>, arch: Option<&String>
) -> i32 {
    /*!
    Call oci-deb to create a debian package from the given OCI
    container tar including flake-ctl app registration hooks
    !*/
    let mut status_code = 255;

    if ! Path::new(defaults::OCIDEB).exists() {
        error!("{} not found, please install the {} package",
            defaults::OCIDEB, defaults::OCIDEB_PACKAGE
        );
        return 1;
    }

    info!("Transforming OCI image to deb...");

    if Path::new(repo).exists() {
        warn!("Repo {} already exists, data gets overwritten or added", repo);
    }

    let mut oci_deb = Command::new(defaults::OCIDEB);
    oci_deb
        .arg("--oci")
        .arg(oci)
        .arg("--repo")
        .arg(repo);

    if ! apps.is_empty() {
        let apps_string = apps.join(",");
        oci_deb
            .arg("--apps")
            .arg(&apps_string);
    }

    if let Some(arch) = arch {
        oci_deb.arg("--arch").arg(arch);
    }

    info!("oci-deb {:?}", oci_deb);

    match oci_deb.output() {
        Ok(output) => {
            status_code = output.status.code().unwrap();
            if ! output.status.success() {
                error!(
                    "Failed, error message(s) reported as: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            } else {
                info!("Successfully created package repository at: {}", repo);
                info!("Following packages are available:");
                let mut packages: Vec<_> = fs::read_dir(repo)
                    .unwrap().map(|r| r.unwrap()).collect();
                packages.sort_by_key(|entry| entry.path());
                for filename in packages {
                    let package = format!("{}", filename.path().display());
                    if package.ends_with(".deb") {
                        info!("--> {}", package);
                    }
                }
                let mut kiwi_repo = String::new();
                kiwi_repo.push_str("\n<repository type=\"apt-deb\"");
                kiwi_repo.push_str(" alias=\"Containers\"");
                kiwi_repo.push_str(" repository_gpgcheck=\"false\">\n");
                kiwi_repo.push_str(
                    &format!("    <source path=\"dir://{}\"/>\n", repo)
                );
                kiwi_repo.push_str("</repository>");
                info!("For use with KIWI add the repo as follows:\n{}",
                    kiwi_repo
                );
            }
        }
        Err(output) => {
            error!("Process terminated({}): {}", defaults::OCIDEB, output)
        }
    }

    status_code
}
