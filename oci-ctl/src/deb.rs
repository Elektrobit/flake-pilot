use std::process::Command;
use std::path::Path;
use std::fs;
use crate::defaults;

pub fn ocideb(
    oci: &String, repo: &String, apps: &Vec<String>, arch: Option<&String>
) -> i32 {
    /*!
    Call oci-deb to create a debian package from the given OCI
    container tar including oci-pilot app registration hooks
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

    if ! arch.is_none() {
        oci_deb
            .arg("--arch")
            .arg(&arch.unwrap());
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
                let mut packages: Vec<_> = fs::read_dir(&repo)
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
