use crate::config::{cfgparse::FlakeCfgVersionParser, itf::FlakeConfig};
use nix::unistd::User;
use serde::Deserialize;
use serde_yaml::Value;
use std::{collections::HashMap, io::Error, path::PathBuf};

use super::itf::{FlakeCfgEngine, FlakeCfgPathProperties, FlakeCfgRuntime, FlakeCfgSetup, FlakeCfgStatic, InstanceMode};

#[derive(Deserialize, Debug)]
struct CfgV1Spec {
    pub(crate) container: Option<CfgV1Container>,
    pub(crate) vm: Option<CfgV1Vm>,
    pub(crate) include: Option<CfgV1Include>,
}

impl CfgV1Spec {
    /// Get container namespace
    fn get_container(&mut self) -> &CfgV1Container {
        if self.container.is_none() {
            self.container = Some(CfgV1Container::default());
        }

        self.container.as_ref().unwrap()
    }

    /// Get VM namespace
    fn get_vm(&mut self) -> &CfgV1Vm {
        if self.vm.is_none() {
            self.vm = Some(CfgV1Vm::default());
        }
        self.vm.as_ref().unwrap()
    }

    /// Get includes namespace
    fn get_includes(&mut self) -> &CfgV1Include {
        if self.include.is_none() {
            self.include = Some(CfgV1Include::default());
        }
        self.include.as_ref().unwrap()
    }
}

/// Container spec
#[derive(Deserialize, Debug)]
struct CfgV1Container {
    pub(crate) name: String,
    target_app_path: String,
    host_app_path: String,
    base_container: Option<String>,
    layers: Option<Vec<String>>,
    runtime: CfgV1OciRuntime,
}

impl CfgV1Container {
    pub(crate) fn default() -> Self {
        CfgV1Container {
            name: "".to_string(),
            target_app_path: "".to_string(),
            host_app_path: "".to_string(),
            base_container: None,
            layers: None,
            runtime: CfgV1OciRuntime::default(),
        }
    }

    /// Name of the container
    fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    /// Exported app path. In config v1 it is assumed `/usr/bin/<pilot-symlink>`.
    fn get_target_app_path(&self) -> &str {
        self.target_app_path.as_ref()
    }

    /// Host app path
    fn get_host_app_path(&self) -> &str {
        self.host_app_path.as_ref()
    }

    /// Optional additional container layers on top of the specified base container
    fn get_layers(&self) -> Option<Vec<String>> {
        self.layers.as_ref()?;
        self.layers.clone()
    }

    /// Optional base container to use with a delta 'container: name'
    fn get_base_container(&self) -> Option<String> {
        self.base_container.as_ref()?;
        Some(self.base_container.clone().unwrap())
    }

    fn get_runtime(&self) -> &CfgV1OciRuntime {
        &self.runtime
    }
}

/// Container spec, runtime
#[derive(Deserialize, Debug)]
struct CfgV1OciRuntime {
    runas: Option<String>,
    resume: Option<bool>,
    attach: Option<bool>,
    podman: Option<Vec<String>>,
}

impl CfgV1OciRuntime {
    pub(crate) fn default() -> Self {
        CfgV1OciRuntime { runas: None, resume: None, attach: None, podman: None }
    }

    fn get_runas_user(&self) -> Option<User> {
        if let Some(luser) = &self.runas {
            if let Ok(luser) = User::from_name(luser) {
                return luser;
            }
        }

        None
    }

    fn has_resume(&self) -> bool {
        self.resume.is_some()
    }

    fn has_attach(&self) -> bool {
        self.attach.is_some()
    }

    /// Get podman runtime. At the time of config v1 was no other runtime support.
    fn get_podman_args(&self) -> Option<Vec<String>> {
        self.podman.to_owned()
    }
}

/// Container spec, includes
#[derive(Deserialize, Debug)]
struct CfgV1Include {
    pub(crate) tar: Option<Vec<String>>,
}

impl CfgV1Include {
    pub(crate) fn default() -> Self {
        CfgV1Include { tar: None }
    }

    fn get_tar(&self) -> Option<Vec<String>> {
        self.tar.to_owned()
    }
}

/// Virtual Machine spec
#[derive(Deserialize, Debug)]
struct CfgV1Vm {
    pub(crate) name: String,
    pub(crate) target_app_path: String,
    pub(crate) host_app_path: String,
    pub(crate) runtime: CfgV1VmRuntime,
}

impl CfgV1Vm {
    pub(crate) fn default() -> Self {
        CfgV1Vm {
            name: "".to_string(),
            target_app_path: "".to_string(),
            host_app_path: "".to_string(),
            runtime: CfgV1VmRuntime { runas: None, resume: None, firecracker: None },
        }
    }

    fn get_name(&self) -> &str {
        self.name.as_ref()
    }

    fn get_target_app_path(&self) -> &str {
        self.target_app_path.as_ref()
    }

    fn get_host_app_path(&self) -> &str {
        self.host_app_path.as_ref()
    }

    fn get_runtime(&self) -> &CfgV1VmRuntime {
        &self.runtime
    }
}

#[derive(Deserialize, Debug)]
struct CfgV1VmRuntime {
    pub(crate) runas: Option<String>,
    pub(crate) resume: Option<bool>,
    pub(crate) firecracker: Option<Value>,
}

impl CfgV1VmRuntime {
    fn get_runas_user(&self) -> Option<User> {
        if let Some(luser) = &self.runas {
            if let Ok(luser) = User::from_name(luser) {
                return luser;
            }
        }

        None
    }

    fn has_resume(&self) -> bool {
        self.resume.is_some() && self.resume.unwrap()
    }

    fn get_firecracker(&self) -> Option<Value> {
        self.firecracker.to_owned()
    }
}

/// Configuration parser, v1.
///
/// It is the original version of the config,
/// prior to 2.2.19 Flakes version.
///
pub struct FlakeCfgV1 {
    content: Value,
}

impl FlakeCfgV1 {
    pub fn new(content: Value) -> Self {
        FlakeCfgV1 { content }
    }

    /// Load configuration for Podman (OCI containers) from the v1 spec.
    ///
    /// Config for v1 essentially supported on that time only podman runtime,
    /// so it is still called "podman config", even though in a theory it
    /// supposed to be a generic OCI containers.
    fn as_container(&self, mut spec: CfgV1Spec) -> FlakeConfig {
        let mut rt_flags = InstanceMode::Volatile;
        if spec.get_container().get_runtime().has_attach() {
            rt_flags |= InstanceMode::Attach;
        }
        if spec.get_container().get_runtime().has_resume() {
            rt_flags |= InstanceMode::Resume;
        }

        let mut paths: HashMap<PathBuf, FlakeCfgPathProperties> = HashMap::new();
        paths.insert(
            PathBuf::from(spec.get_container().get_target_app_path()),
            FlakeCfgPathProperties::new(PathBuf::from(spec.get_container().get_host_app_path())),
        );

        FlakeConfig {
            version: 1,
            runtime: FlakeCfgRuntime {
                image_name: spec.get_container().get_name().to_string(),
                base_layer: spec.get_container().get_base_container(),
                layers: spec.get_container().get_layers(),
                run_as: spec.get_container().get_runtime().get_runas_user(),
                instance_mode: rt_flags,
                paths,
            },
            engine: FlakeCfgEngine {
                pilot: "podman".to_string(),
                args: spec.get_container().get_runtime().get_podman_args(),

                // No known params of OCI containers at this point
                params: None,
            },
            static_data: FlakeCfgStatic { bundles: spec.get_includes().get_tar() },
            setup: FlakeCfgSetup {},
        }
    }

    /// Load configuration for the Virtual Machine from the v1 spec.
    fn as_vm(&self, mut spec: CfgV1Spec) -> FlakeConfig {
        let mut rt_flags = InstanceMode::Volatile;
        if spec.get_vm().get_runtime().has_resume() {
            rt_flags |= InstanceMode::Resume;
        }

        let mut paths: HashMap<PathBuf, FlakeCfgPathProperties> = HashMap::new();
        paths.insert(
            PathBuf::from(spec.get_vm().get_target_app_path()),
            FlakeCfgPathProperties::new(PathBuf::from(spec.get_vm().get_host_app_path())),
        );

        FlakeConfig {
            version: 1,
            runtime: FlakeCfgRuntime {
                image_name: spec.get_vm().get_name().to_string(),
                base_layer: None,
                layers: None,
                run_as: spec.get_vm().get_runtime().get_runas_user(),
                instance_mode: rt_flags,
                paths,
            },
            engine: FlakeCfgEngine {
                pilot: "firecracker".to_string(),
                args: None,
                params: spec.get_vm().get_runtime().get_firecracker(),
            },
            static_data: FlakeCfgStatic { bundles: None },
            setup: FlakeCfgSetup {},
        }
    }
}

impl FlakeCfgVersionParser for FlakeCfgV1 {
    fn parse(&self) -> Result<FlakeConfig, Error> {
        match serde_yaml::from_value::<CfgV1Spec>(self.content.to_owned()) {
            Ok(spec) => {
                if let Value::Mapping(content) = &self.content {
                    if content.contains_key("container") {
                        return Ok(self.as_container(spec));
                    } else if content.contains_key("vm") {
                        return Ok(self.as_vm(spec));
                    }
                }
            }
            Err(err) => return Err(Error::new(std::io::ErrorKind::InvalidData, format!("Config parse error: {}", err))),
        }

        Err(Error::new(std::io::ErrorKind::InvalidData, "Unknown config scheme"))
    }
}
