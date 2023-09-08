use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_yaml::Value;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config<R> {
    /// The unique name of this flake
    pub name: String,
    /// Location of the coresponding pilot link
    pub host_path: PathBuf,

    #[serde(default)]
    pub include: Vec<String>,

    // Engine section
    /// What engine is this flake meant for
    ///
    /// A flake should be run with "`<engine>-pilot`"
    ///
    /// If engine is `None` this flake should not be run at all
    pub engine: Option<String>,
    /// The runtime specific options for this engine
    #[serde(default)]
    pub runtime: R,
}

pub trait Runtime {
    fn engine() -> Option<String>;
}

type GenericValue = serde_yaml::Value;

/// A generic flake config that collects the runtime value into a hashmap
///
/// Meant for use in utility applications like "flake-ctl list"
///
/// **Note** The values in `runtime` are only guaranteed to be valid YAML, they might not represent a valid configuration
/// for the specified `engine`
pub type FlakeConfig = Config<GenericValue>;

impl<R: Runtime + Serialize> Config<R> {
    /// Change the runtime of the flake
    ///
    /// **Note:** This will not migrate compatible options or update the symlink on disk,
    /// it will simply drop any associated runtime information and replace the name of the engine
    pub fn with_runtime<New: Runtime>(self, runtime: New) -> Config<New> {
        let Self { name, host_path, include, .. } = self;
        Config { name, host_path, engine: New::engine(), runtime, include }
    }

    pub fn into_generic(self) -> FlakeConfig {
        let Self { name, host_path, include, engine, runtime } = self;
        Config { name, host_path, engine, runtime: serde_yaml::to_value(runtime).expect("Can always be converted"), include }
    }
}

impl Runtime for GenericValue {
    fn engine() -> Option<String> {
        None
    }
}

#[derive(Debug, Error)]
pub enum ConfigConvertionError {
    #[error("The engines of the runtime and the generic config must be the same")]
    WrongEngine,
    #[error(transparent)]
    SyntaxError(#[from] serde_yaml::Error),
}

impl FlakeConfig {
    /// Try to convert the Config into a config for the given runtime
    pub fn try_conversion<R: Runtime + DeserializeOwned>(self) -> Result<Config<R>, ConfigConvertionError> {
        if R::engine() != self.engine {
            return Err(ConfigConvertionError::WrongEngine);
        }
        let new_runtime: R = serde_yaml::from_value(self.runtime.clone())?;
        Ok(self.with_runtime(new_runtime))
    }
}

/// Identical to Config<GenericValue> except all fields are optional
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PartialCofig {
    /// The unique name of this flake
    pub name: Option<String>,
    /// Location of the coresponding pilot link
    pub host_path: Option<PathBuf>,

    #[serde(default)]
    pub include: Vec<String>,

    // Engine section
    /// What engine is this flake meant for
    ///
    /// A flake should be run with "`<engine>-pilot`"
    ///
    /// If engine is `None` this flake should not be run at all
    pub engine: Option<String>,
    /// The runtime specific options for this engine
    #[serde(default)]
    pub runtime: GenericValue,

    /// An ordered list of all configs that have been merged into this one
    merged: Vec<String>,
}

#[derive(Debug, Error)]
pub struct ConfigMergeError {
    kind: MergeErrorKind,
    configs: Vec<String>,
}

impl Display for ConfigMergeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.kind, f)?;
        f.write_str(&format!("; Combined files: {:?}", self.configs))
    }
}

#[derive(Debug, Error)]
pub enum MergeErrorKind {
    #[error("No name was given")]
    MissingName,
    #[error("No path to the host app was given")]
    MissingHostPath,
    #[error(transparent)]
    Conversion(#[from] ConfigConvertionError),
}

impl PartialCofig {
    /// Turn this `PartialConfig` into a `Config<R>`
    pub fn finish<R: Runtime + DeserializeOwned>(self) -> Result<Config<R>, ConfigMergeError> {
        Ok(Config {
            name: self.name.ok_or(ConfigMergeError { kind: MergeErrorKind::MissingName, configs: self.merged.clone() })?,
            host_path: self
                .host_path
                .ok_or(ConfigMergeError { kind: MergeErrorKind::MissingHostPath, configs: self.merged.clone() })?,
            include: self.include,
            engine: self.engine,
            runtime: self.runtime,
        }
        .try_conversion()
        .map_err(|err| ConfigMergeError { kind: err.into(), configs: self.merged })?)
    }

    /// Update this `PartialConfig` with another, consuming both.
    /// 
    /// The values from the other `PartialConfig` take precedence unless they are `null`/`None`.
    /// 
    /// `Mappings` are combined recursively, all other values will be replaced.
    /// 
    /// The lists of combined configs of the other `PartialConfig` are appended to this ones
    pub fn update(mut self, other: PartialCofig) -> Self {
        self.name = other.name.or(self.name);
        self.host_path = other.host_path.or(self.host_path);
        self.engine = other.engine.or(self.engine);
        self.merged.extend(other.merged);
        self.runtime = Self::merge_values(self.runtime, other.runtime);
        self
    }

    fn merge_values(base: Value, update: Value) -> Value {
        let result = match (base, update) {
            (Value::Mapping(mut base), Value::Mapping(update)) => {
                // TODO: This could be written nicer by somebody who wants to fight with the lifetimes of `Mapping::entry`
                for (key, value) in update {
                    let old = base.get(&key).cloned().unwrap_or_default();
                    base.insert(key, Self::merge_values(old, value));
                }
                base.into()
            },
            (base, Value::Null) => base,
            (_, update) => update
        };
        result
    }
}

#[cfg(test)]
mod test {
    use crate::config::*;
    use serde::{Deserialize, Serialize};
    use serde_yaml::Value;
    use std::{path::PathBuf, vec};

    #[derive(Debug, Deserialize, Serialize)]
    struct PodmanRuntime {
        base: String,
        #[serde(default)]
        layers: Vec<String>,
        /// The app to run inside the engine
        target: Option<PathBuf>,
        #[serde(default)]
        resume: bool,
        #[serde(default)]
        attach: bool,
        runas: Option<String>,
        args: Vec<String>,
    }

    impl Runtime for PodmanRuntime {
        fn engine() -> Option<String> {
            Some("podman".to_owned())
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct FirecrackerRuntime {
        base: String,
        #[serde(default)]
        layers: Vec<String>,
        /// The app to run inside the engine
        target: Option<PathBuf>,
        #[serde(default)]
        resume: bool,
        vm: FirecrackerVMSettings,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct FirecrackerVMSettings {
        #[serde(default)]
        boot_args: Vec<String>,
        overlay_size: String,
        rootfs_image_path: Option<PathBuf>,
        kernel_image_path: Option<PathBuf>,
        initrd_path: Option<PathBuf>,
        mem_size_mib: usize,
        vcpu_count: Option<usize>,
        cache_type: Option<String>,
    }

    impl Runtime for FirecrackerRuntime {
        fn engine() -> Option<String> {
            Some("firecracker".to_owned())
        }
    }

    #[test]
    fn test_type_conversions() {
        let podman = Config {
            name: "my_podman_flake".to_owned(),
            host_path: PathBuf::from("/usr/bin/my_flake"),
            include: vec![],
            engine: Some("podman".to_owned()),
            runtime: PodmanRuntime {
                base: "ubuntu".to_owned(),
                target: None,
                layers: vec![],
                resume: false,
                attach: false,
                runas: None,
                args: vec!["-ti".to_owned()],
            },
        };

        let firecracker = Config {
            name: "my_firecracker_flake".to_owned(),
            host_path: PathBuf::from("/usr/bin/my_flake"),
            include: vec![],
            engine: Some("firecracker".to_owned()),
            runtime: FirecrackerRuntime {
                base: "ubuntu".to_owned(),
                layers: vec![],
                target: None,
                resume: false,
                vm: FirecrackerVMSettings {
                    boot_args: vec![
                        "init=/usr/sbin/sci".to_owned(),
                        "console=ttyS0".to_owned(),
                        "root=/dev/vda".to_owned(),
                        "acpi=off".to_owned(),
                        "rd.neednet=1".to_owned(),
                        "ip=dhcp".to_owned(),
                        "quiet".to_owned(),
                    ],
                    overlay_size: "2GiB".to_owned(),
                    rootfs_image_path: None,
                    kernel_image_path: None,
                    initrd_path: None,
                    mem_size_mib: 4096,
                    vcpu_count: None,
                    cache_type: None,
                },
            },
        };

        println!("{}", serde_yaml::to_string(&podman).unwrap());
        println!("{}", serde_yaml::to_string(&firecracker).unwrap());
        // println!("{}", serde_yaml::to_string(&podman.into_generic()).unwrap());
        // println!("{}", serde_yaml::to_string(&firecracker.into_generic()).unwrap());

        let as_generic: FlakeConfig = serde_yaml::from_str(&serde_yaml::to_string(&firecracker).unwrap()).unwrap();
        println!("{}", serde_yaml::to_string(&as_generic).unwrap());
        println!("{}", as_generic.try_conversion::<FirecrackerRuntime>().unwrap().name);

        let _x = vec![podman.into_generic(), firecracker.into_generic()];
    }

    #[test]
    fn test_combine_podman() {
        let base = PartialCofig {
            name: Some("Base".to_owned()),
            engine: Some("podman".to_owned()),
            merged: vec!["base.yaml".to_owned()],
            ..Default::default()
        };

        let extension_1 = PartialCofig {
            host_path: Some(PathBuf::from("/path/to/the/app")),            
            merged: vec!["extension_1.yaml".to_owned()],
            runtime: Value::Mapping([
                ("base".into(), "ubuntu".into()),
                ("resume".into(), Value::Null),
                ("attach".into(), Value::Bool(true)),
                ("runas".into(), "someuser".into()),
                ("args".into(), Value::Null),
            ].into_iter().collect()),
            ..Default::default()
        };

        let extension_2 = PartialCofig {
            host_path: Some(PathBuf::from("/path/to/the/app")),            
            merged: vec!["extension_2.yaml".to_owned()],
            runtime: Value::Mapping([
                ("resume".into(), Value::Bool(true)),
                ("attach".into(), Value::Bool(true)),
                ("runas".into(), Value::Null),
                ("args".into(), Value::Sequence(Default::default())),
            ].into_iter().collect()),
            ..Default::default()
        };

        let result = base;
        let result = result.update(extension_1);
        let result = result.update(extension_2);

        let result = result.finish::<PodmanRuntime>().unwrap();
        assert_eq!(result.runtime.attach, true);
        assert_eq!(result.runtime.resume, true);
        assert_eq!(result.runtime.base, "ubuntu".to_owned());
        assert_eq!(result.runtime.runas, Some("someuser".to_owned()));
        assert_eq!(result.runtime.args, Vec::<String>::new());
    }


    #[test]
    fn test_combine_firecracker() {
        let base = PartialCofig {
            name: Some("Base".to_owned()),
            engine: Some("firecracker".to_owned()),
            merged: vec!["base.yaml".to_owned()],
            ..Default::default()
        };

        let extension_1 = PartialCofig {
            host_path: Some(PathBuf::from("/path/to/the/app")),            
            merged: vec!["extension_1.yaml".to_owned()],
            runtime: Value::Mapping([
                ("base".into(), "ubuntu".into()),
                ("resume".into(), Value::Null),
                ("runas".into(), "someuser".into()),
                ("vm".into(), Value::Mapping([
                    ("mem_size_mib".into(), 1024.into()),
                ].into_iter().collect())),
            ].into_iter().collect()),
            ..Default::default()
        };

        let extension_2 = PartialCofig {
            host_path: Some(PathBuf::from("/path/to/the/app")),            
            merged: vec!["extension_2.yaml".to_owned()],
            runtime: Value::Mapping([
                ("resume".into(), Value::Bool(true)),
                ("runas".into(), Value::Null),
                ("args".into(), Value::Sequence(Default::default())),
                ("vm".into(), Value::Mapping([
                    ("mem_size_mib".into(), 4096.into()),
                    ("overlay_size".into(), "2MiB".into()),
                ].into_iter().collect())),
            ].into_iter().collect()),
            ..Default::default()
        };

        let result = base;
        let result = result.update(extension_1);
        let result = result.update(extension_2);

        let result = result.finish::<FirecrackerRuntime>().unwrap();
        assert_eq!(result.runtime.resume, true);
        assert_eq!(result.runtime.vm.mem_size_mib, 4096);
        assert_eq!(result.runtime.vm.overlay_size, "2MiB");
    }

}
