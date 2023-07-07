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
use crate::defaults;
use serde::{Deserialize, Serialize};
use serde_yaml::{self};
use std::io::{Error, ErrorKind};
use std::path::Path;

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

// AppConfig represents application yaml configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub include: AppInclude,
    pub container: Option<AppContainer>,
    pub vm: Option<AppFireCracker>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppContainer {
    pub name: String,
    pub target_app_path: String,
    pub host_app_path: String,
    pub base_container: Option<String>,
    pub layers: Option<Vec<String>>,
    pub runtime: Option<AppContainerRuntime>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppContainerRuntime {
    pub runas: Option<String>,
    pub resume: Option<bool>,
    pub attach: Option<bool>,
    pub podman: Option<Vec<String>>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppInclude {
    pub tar: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppFireCracker {
    pub name: String,
    pub target_app_path: String,
    pub host_app_path: String,
    pub base_vm: Option<String>,
    pub layers: Option<Vec<String>>,
    pub runtime: Option<AppFireCrackerRuntime>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppFireCrackerRuntime {
    pub runas: Option<String>,
    pub resume: Option<bool>,
    pub firecracker: Option<AppFireCrackerEngine>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct AppFireCrackerEngine {
    pub boot_args: Option<Vec<String>>,
    pub overlay_size: Option<String>,
    pub rootfs_image_path: Option<String>,
    pub kernel_image_path: Option<String>,
    pub initrd_path: Option<String>,
    pub mem_size_mib: Option<i32>,
    pub vcpu_count: Option<i32>,
    pub cache_type: Option<String>,
}

impl AppConfig {
    #[allow(clippy::too_many_arguments)]
    pub fn save_container(
        config_file: &Path,
        container: &str,
        target_app_path: &str,
        host_app_path: &str,
        base: Option<&String>,
        layers: Option<Vec<String>>,
        includes_tar: Option<Vec<String>>,
        resume: bool,
        attach: bool,
        run_as: Option<&String>,
        opts: Option<Vec<String>>,
    ) -> Result<(), GenericError> {
        /*!
        save stores an AppConfig to the given file
        !*/
        let template = std::fs::File::open(defaults::FLAKE_TEMPLATE_CONTAINER)
            .unwrap_or_else(|_| panic!("Failed to open {}", defaults::FLAKE_TEMPLATE_CONTAINER));
        let mut yaml_config: AppConfig =
            serde_yaml::from_reader(template).expect("Failed to import config template");
        let container_config = yaml_config.container.as_mut().unwrap();

        container_config.name = container.to_owned();
        container_config.target_app_path = target_app_path.to_owned();
        container_config.host_app_path = host_app_path.to_owned();

        container_config.base_container = base.cloned();
        container_config.layers = layers;
        yaml_config.include.tar = includes_tar;

        if let Some(runtime) = container_config.runtime.as_mut() {
            if resume {
                runtime.resume = Some(resume);
            } else if attach {
                runtime.attach = Some(attach);
            } else {
                // default: remove the container if no resume/attach is set
                runtime.podman.as_mut().unwrap().push("--rm".to_string());
            }

            runtime.runas = run_as.cloned();

            runtime.podman = opts.map(|x| {
                x.iter()
                    .map(|x| x.trim_start_matches('\\').to_owned())
                    .collect()
            });
        }

        let config = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(config_file)
            .unwrap_or_else(|_| panic!("Failed to open {:?}", config_file));
        serde_yaml::to_writer(config, &yaml_config).unwrap();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_vm(
        config_file: &Path,
        vm: &String,
        target_app_path: &str,
        host_app_path: &String,
        run_as: Option<&String>,
        overlay_size: Option<&String>,
        no_net: bool,
        resume: bool,
        includes_tar: Option<Vec<String>>,
    ) -> Result<(), GenericError> {
        /*!
        save stores an AppConfig to the given file
        !*/
        let image_dir = format!("{}/{}", defaults::FIRECRACKER_IMAGES_DIR, vm);
        let template = std::fs::File::open(defaults::FLAKE_TEMPLATE_FIRECRACKER)
            .unwrap_or_else(|_| panic!("Failed to open {}", defaults::FLAKE_TEMPLATE_FIRECRACKER));
        let mut yaml_config: AppConfig =
            serde_yaml::from_reader(template).expect("Failed to import config template");
        let vm_config = yaml_config.vm.as_mut().unwrap();

        vm_config.name = vm.clone();
        vm_config.target_app_path = target_app_path.to_owned();
        vm_config.host_app_path = host_app_path.to_owned();

        yaml_config.include.tar = includes_tar;

        if let Some(runtime) = vm_config.runtime.as_mut() {
            if resume {
                runtime.resume = Some(resume);
            }

            runtime.runas = run_as.cloned();

            if let Some(firecracker) = runtime.firecracker.as_mut() {
                firecracker.overlay_size = overlay_size.cloned();

                let rootfs_image_path =
                    format!("{}/{}", image_dir, defaults::FIRECRACKER_ROOTFS_NAME);

                if Path::new(&rootfs_image_path).exists() {
                    firecracker.rootfs_image_path = Some(rootfs_image_path);
                } else {
                    return Err(Box::new(Error::new(
                        ErrorKind::NotFound,
                        format!("No rootfs image found: {}", rootfs_image_path),
                    )));
                }
                let kernel_image_path =
                    format!("{}/{}", image_dir, defaults::FIRECRACKER_KERNEL_NAME);

                if Path::new(&kernel_image_path).exists() {
                    firecracker.kernel_image_path = Some(kernel_image_path);
                } else {
                    return Err(Box::new(Error::new(
                        ErrorKind::NotFound,
                        format!("No kernel image found: {}", kernel_image_path),
                    )));
                }

                let initrd_path = format!("{}/{}", image_dir, defaults::FIRECRACKER_INITRD_NAME);
                if Path::new(&initrd_path).exists() {
                    firecracker.initrd_path = Some(initrd_path);
                }

                if let Some(boot_args) = firecracker.boot_args.as_mut() {
                    if no_net {
                        boot_args.retain(|arg| !arg.starts_with("ip="))
                    }
                    if resume {
                        boot_args.push("sci_resume=1".to_owned());
                    }
                }
            }
        }

        let config = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(config_file)
            .unwrap_or_else(|_| panic!("Failed to open {:?}", config_file));
        serde_yaml::to_writer(config, &yaml_config).unwrap();
        Ok(())
    }

    pub fn init_from_file(config_file: &Path) -> Result<AppConfig, GenericError> {
        /*!
        new creates the new AppConfig class by reading and
        deserializing the data from a given yaml configuration
        !*/
        let config = std::fs::File::open(config_file)
            .unwrap_or_else(|_| panic!("Failed to open {:?}", config_file));
        let yaml_config: AppConfig =
            serde_yaml::from_reader(config).expect("Failed to import config file");
        Ok(yaml_config)
    }
}
