use super::itf::{FlakeCfgEngine, FlakeCfgPathProperties, FlakeCfgRuntime, FlakeCfgSetup, FlakeCfgStatic, InstanceMode, PathMap};
use crate::config::{cfgparse::FlakeCfgVersionParser, itf::FlakeConfig};
use nix::unistd::User;
use serde::Deserialize;
use serde_yaml::Value;
use std::{collections::HashMap, io::Error, path::PathBuf};

#[derive(Deserialize, Debug)]
struct CfgV2Spec {
    version: u8,
    runtime: CfgV2Runtime,
    engine: CfgV2Engine,

    #[serde(rename = "static")]
    static_data: Option<Vec<String>>,
}

/// Runtime section
#[derive(Deserialize, Debug)]
struct CfgV2Runtime {
    name: String,
    path_map: HashMap<String, Value>,
    base_layer: Option<String>,
    layers: Option<Vec<String>>,
    user: Option<String>,
    instance: Option<String>,
}

impl CfgV2Runtime {
    fn get_runas_user(&self, user: Option<String>) -> Option<User> {
        if let Some(luser) = if user.is_some() { user } else { self.user.to_owned() } {
            if let Ok(luser) = User::from_name(&luser) {
                return luser;
            }
        }

        None
    }

    fn get_instance(&self) -> InstanceMode {
        let mut im = InstanceMode::Volatile;
        if let Some(mode) = self.instance.to_owned() {
            for m in mode.split(' ') {
                match m {
                    "resume" => {
                        im |= InstanceMode::Resume;
                    }
                    "attach" => {
                        im |= InstanceMode::Attach;
                    }
                    &_ => {}
                }
            }
        }

        im
    }

    fn get_path_map(&self) -> PathMap {
        let mut pmap: PathMap = PathMap::default();
        self.path_map.clone().into_iter().for_each(|(target, props)| {
            if let Ok(rp) = serde_yaml::from_value::<CfgV2PathProperties>(props) {
                let mut i_mode = InstanceMode::Volatile;
                if let Some(instance) = rp.instance {
                    for mut mode in instance.split(' ') {
                        mode = mode.trim();
                        match mode {
                            "attach" => {
                                i_mode |= InstanceMode::Attach;
                            }
                            "resume" => {
                                i_mode |= InstanceMode::Resume;
                            }
                            _ => {}
                        }
                    }
                } else {
                    i_mode = self.get_instance();
                }
                pmap.inner.insert(
                    PathBuf::from(target.clone()),
                    FlakeCfgPathProperties {
                        exports: if rp.exports.is_none() { PathBuf::from(target) } else { PathBuf::from(rp.exports.unwrap()) },
                        run_as: if rp.user.is_some() { self.get_runas_user(rp.user) } else { None },
                        instance_mode: Some(i_mode),
                    },
                );
            }
        });

        pmap
    }
}

#[derive(Deserialize, Debug)]
struct CfgV2PathProperties {
    exports: Option<String>,
    user: Option<String>,
    instance: Option<String>,
}

///Engine section
#[derive(Deserialize, Debug)]
struct CfgV2Engine {
    pub(crate) pilot: String,
    pub(crate) args: Option<Vec<String>>,
    pub(crate) params: Option<Value>,
}

/// Configuration parser, v2.
pub struct FlakeCfgV2 {
    content: Value,
}

impl FlakeCfgV2 {
    pub fn new(content: Value) -> Self {
        FlakeCfgV2 { content }
    }

    fn as_cfg(&self, spec: CfgV2Spec) -> FlakeConfig {
        FlakeConfig {
            version: spec.version,
            runtime: FlakeCfgRuntime {
                image_name: spec.runtime.name.to_owned(),
                base_layer: spec.runtime.base_layer.to_owned(),
                layers: spec.runtime.layers.to_owned(),
                run_as: spec.runtime.get_runas_user(None),
                instance_mode: spec.runtime.get_instance(),
                paths: spec.runtime.get_path_map(),
            },
            engine: FlakeCfgEngine { pilot: spec.engine.pilot, args: spec.engine.args, params: spec.engine.params },
            static_data: FlakeCfgStatic { bundles: spec.static_data },
            setup: FlakeCfgSetup {},
        }
    }
}

impl FlakeCfgVersionParser for FlakeCfgV2 {
    fn parse(&self) -> Result<FlakeConfig, Error> {
        match serde_yaml::from_value::<CfgV2Spec>(self.content.to_owned()) {
            Ok(spec) => Ok(self.as_cfg(spec)),
            Err(err) => Err(Error::new(std::io::ErrorKind::InvalidData, format!("Config v2 parse error: {}", err))),
        }
    }
}
