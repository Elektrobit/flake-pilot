use std::{
    error::Error,
    fmt::{Debug, Display},
    fs::{self, OpenOptions},
    marker::PhantomData,
    path::{Path, PathBuf},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_yaml::Value;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Config<R> {
    /// The unique name of this flake
    pub name: String,
    /// Location of the corresponding pilot link
    pub host_path: PathBuf,

    #[serde(default)]
    pub include: Vec<String>,

    // Engine section
    /// What type of engine is this flake meant for, e.g. "container" or "vm"
    pub engine_type: String,
    /// The runtime specific options for this engine
    pub runtime: R,
}

pub trait EngineConfig {
    /// The type of engine, e.g. "container" or "vm"
    fn engine_type() -> String;
    /// The specific engine to be used, e.g. "podman", "runc", "firecracker"
    fn engine() -> Option<String>;
}

/// Mark that this config is meant for a specific engine i.e. the return value of `EngineConfig::engine()` is always `Some(_)`
pub trait ConcreteEngine: EngineConfig {
    fn concrete_engine() -> String {
        Self::engine().unwrap()
    }
}

type GenericValue = serde_yaml::Value;

/// A generic flake config that collects the runtime value into a hashmap
///
/// Meant for use in utility applications like "flake-ctl list"
///
/// **Note** The values in `runtime` are only guaranteed to be valid YAML, they might not represent a valid configuration
/// for the specified `engine`
pub type FlakeConfig = Config<GenericValue>;

impl<R> Config<R> {
    /// Change the runtime of the flake
    ///
    /// **Note:** This will not migrate compatible options or update the symlink on disk,
    /// it will simply drop any associated runtime information and replace the name of the engine
    pub fn with_runtime<New: EngineConfig>(self, runtime: New) -> Config<New> {
        let Self { name, host_path, include, .. } = self;
        Config { name, host_path, engine_type: New::engine_type(), runtime, include }
    }
}

impl<Runtime: Serialize> Config<Runtime> {
    pub fn raw(&self) -> serde_yaml::Result<String> {
        serde_yaml::to_string(self)
    }
}

#[derive(Debug, Error)]
pub enum ConfigConversionError {
    #[error("This config is for a \"{}\"-type engine, a \"{}\"-type is required", .0, .1)]
    WrongEngineType(String, String),
    #[error(transparent)]
    SyntaxError(#[from] serde_yaml::Error),
}

impl FlakeConfig {
    /// Try to convert the Config into a config for the given runtime by combining the engine-type and engine entries in the runtime mapping
    ///
    /// Engine specific settings take precedent over type settings
    ///
    /// Engines may specify settings not present in the type settings
    pub fn try_conversion<R: EngineConfig + DeserializeOwned>(self) -> Result<Config<R>, ConfigConversionError> {
        if R::engine_type() != self.engine_type {
            return Err(ConfigConversionError::WrongEngineType(R::engine_type(), self.engine_type));
        }
        let type_cfg = self.runtime.get(&self.engine_type).cloned().unwrap_or(Value::Null);
        let engine_cfg = self.runtime.get(R::engine().unwrap_or_default()).cloned().unwrap_or(Value::Null);

        Ok(self.with_runtime(serde_yaml::from_value(merge_values(type_cfg, engine_cfg))?))
    }
}

pub struct AnyContainer;
impl EngineConfig for AnyContainer {
    fn engine() -> Option<String> {
        None
    }

    fn engine_type() -> String {
        "container".to_owned()
    }
}

pub struct AnyVm;
impl EngineConfig for AnyVm {
    fn engine() -> Option<String> {
        None
    }

    fn engine_type() -> String {
        "vm".to_owned()
    }
}

/// Identical to Config<GenericValue> except all fields are optional
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PartialConfig {
    /// The unique name of this flake
    pub name: Option<String>,
    /// Location of the corresponding pilot link
    pub host_path: Option<PathBuf>,

    #[serde(default)]
    pub include: Vec<String>,

    // Engine section
    /// What engine is this flake meant for
    ///
    /// A flake should be run with "`<engine>-pilot`"
    ///
    /// If engine is `None` this flake should not be run at all
    pub engine_type: Option<String>,
    /// The runtime specific options for this engine
    #[serde(default)]
    pub runtime: GenericValue,

    /// An ordered list of all configs that have been merged into this one
    #[serde(default)]
    merged: Vec<String>,
}

#[derive(Default)]
pub struct ConfigBuilder<Engine: EngineConfig> {
    inner: PartialConfig,
    _engine: PhantomData<Engine>,
}

impl<Engine: EngineConfig> ConfigBuilder<Engine> {
    pub fn new() -> Self {
        Self { inner: PartialConfig { engine_type: Some(Engine::engine_type()), ..Default::default() }, _engine: PhantomData }
    }

    pub fn build(self) -> PartialConfig {
        self.inner
    }

    pub fn name(mut self, name: &str) -> Self {
        self.inner.name = Some(name.to_owned());
        self
    }

    pub fn host_path(mut self, path: impl AsRef<Path>) -> Self {
        self.inner.host_path = Some(path.as_ref().to_owned());
        self
    }

    pub fn add_include(mut self, include: &str) -> Self {
        self.inner.include.push(include.to_owned());
        self
    }

    /// Set a value in the engine_type section of the config
    // TODO: Not happy with the function name
    pub fn set_in_type(self, key: &str, value: impl Into<GenericValue>) -> Self {
        self.set_by_engine(&Engine::engine_type(), key, value)
    }

    /// Directly set the given value for the given engine
    pub fn set_by_engine(mut self, engine: &str, key: &str, value: impl Into<GenericValue>) -> Self {
        self.inner.runtime[engine][key] = value.into();
        self
    }

    pub fn configure<New: EngineConfig>(self) -> ConfigBuilder<New> {
        ConfigBuilder { inner: self.inner, _engine: PhantomData }
    }
}

impl<Engine: ConcreteEngine> ConfigBuilder<Engine> {
    /// Set the value for the current engine
    pub fn set(self, key: &str, value: impl Into<GenericValue>) -> Self {
        self.set_by_engine(&Engine::concrete_engine(), key, value)
    }
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
    #[error("No engine type was given")]
    MissingEngineType,
    #[error(transparent)]
    Conversion(#[from] ConfigConversionError),
}

impl PartialConfig {
    /// Turn this `PartialConfig` into a `Config<R>`
    pub fn finish<R: EngineConfig + DeserializeOwned>(self) -> Result<Config<R>, ConfigMergeError> {
        Config {
            name: self.name.ok_or(ConfigMergeError { kind: MergeErrorKind::MissingName, configs: self.merged.clone() })?,
            host_path: self
                .host_path
                .ok_or(ConfigMergeError { kind: MergeErrorKind::MissingHostPath, configs: self.merged.clone() })?,
            include: self.include,
            engine_type: self
                .engine_type
                .ok_or(ConfigMergeError { kind: MergeErrorKind::MissingHostPath, configs: self.merged.clone() })?,
            runtime: self.runtime,
        }
        .try_conversion()
        .map_err(|err| ConfigMergeError { kind: err.into(), configs: self.merged })
    }

    /// Update this `PartialConfig` with another, consuming both.
    ///
    /// The values from the other `PartialConfig` take precedence unless they are `null`/`None`.
    ///
    /// `Mappings` are combined recursively, all other values will be replaced.
    ///
    /// The lists of combined configs of the other `PartialConfig` are appended to this ones
    pub fn update(mut self, other: PartialConfig) -> Self {
        self.name = other.name.or(self.name);
        self.host_path = other.host_path.or(self.host_path);
        self.engine_type = other.engine_type.or(self.engine_type);
        self.merged.extend(other.merged);
        self.runtime = merge_values(self.runtime, other.runtime);
        self
    }
}

fn merge_values(base: Value, update: Value) -> Value {
    match (base, update) {
        (Value::Mapping(mut base), Value::Mapping(update)) => {
            // TODO: This could be written nicer by somebody who wants to fight with the lifetimes of `Mapping::entry`
            for (key, value) in update {
                let old = base.get(&key).cloned().unwrap_or_default();
                base.insert(key, merge_values(old, value));
            }
            base.into()
        }
        (base, Value::Null) => base,
        (_, update) => update,
    }
}

/// Load the config for a given flake
// TODO: return FlakeError once that is in common
pub fn load_config<R: EngineConfig + DeserializeOwned>(path: impl AsRef<Path>) -> Result<Config<R>, Box<dyn Error>> {
    let path = PathBuf::from(path.as_ref());
    let mut base: PartialConfig = serde_yaml::from_reader(OpenOptions::new().read(true).open(path.with_extension("yaml"))?)?;
    // TODO stabilize alphanumeric sorting
    for entry in fs::read_dir(path.with_extension("d"))? {
        let other = serde_yaml::from_reader(OpenOptions::new().read(true).open(entry?.path())?)?;
        base = base.update(other);
    }
    Ok(base.finish()?)
}

#[cfg(test)]
mod test {
    use crate::config::*;
    use serde::{Deserialize, Serialize};

    use std::path::PathBuf;

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
        #[serde(default)]
        args: Vec<String>,
    }

    impl EngineConfig for PodmanRuntime {
        fn engine_type() -> String {
            "container".to_owned()
        }
        fn engine() -> Option<String> {
            Some("podman".to_owned())
        }
    }

    impl ConcreteEngine for PodmanRuntime {}

    #[derive(Debug, Deserialize, Serialize)]
    struct FirecrackerRuntime {
        base: String,
        #[serde(default)]
        layers: Vec<String>,
        /// The app to run inside the engine
        target: Option<PathBuf>,
        #[serde(default)]
        resume: bool,
        #[serde(flatten)]
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

    impl EngineConfig for FirecrackerRuntime {
        fn engine_type() -> String {
            "vm".to_owned()
        }
        fn engine() -> Option<String> {
            Some("firecracker".to_owned())
        }
    }

    #[test]
    fn test_combine_podman() {
        let base: PartialConfig = serde_yaml::from_str(
            r#"
        name: Base
        engine_type: container
        engine: podman
        "#,
        )
        .unwrap();

        let extension_1: PartialConfig = serde_yaml::from_str(
            r#"
        host_path: path/to/the/app
        runtime:
            container:
                base: ubuntu
                runas: someuser
                resume: ~
                attach: true
            podman:
                args: ~
            docker:
                args: ~
        "#,
        )
        .unwrap();

        let extension_2: PartialConfig = serde_yaml::from_str(
            r#"
        host_path: path/to/the/app
        runtime:
            podman:
                resume: true
                attach: true
                runas: ~
                args: ~
        "#,
        )
        .unwrap();

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
        let base: PartialConfig = serde_yaml::from_str(
            r#"
        name: Base
        host_path: path/to/the/app
        engine_type: vm
        "#,
        )
        .unwrap();

        let extension_1: PartialConfig = serde_yaml::from_str(
            r#"
        host_path: path/to/the/app
        runtime:
            vm:
                base: ubuntu
                resume: ~
                runas: ~
                mem_size_mib: 1024
        "#,
        )
        .unwrap();

        let extension_2: PartialConfig = serde_yaml::from_str(
            r#"
        host_path: path/to/the/app
        runtime:
            vm:
                base: ubuntu
                resume: true
                runas: ~
                mem_size_mib: 4096
                overlay_size: 2MiB
        "#,
        )
        .unwrap();

        let result = base;
        let result = result.update(extension_1);
        let result = result.update(extension_2);

        let result = result.finish::<FirecrackerRuntime>().unwrap();
        assert_eq!(result.runtime.resume, true);
        assert_eq!(result.runtime.vm.mem_size_mib, 4096);
        assert_eq!(result.runtime.vm.overlay_size, "2MiB");
    }

    #[test]
    fn constructing_partial() {
        let result = ConfigBuilder::<PodmanRuntime>::new()
            .name("my_flake")
            .host_path("/bin/ein/socio/path")
            .set("attach", true)
            .set("resume", true)
            .set("base", "ubuntu")
            .build()
            .finish::<PodmanRuntime>()
            .unwrap();

        assert_eq!(result.runtime.attach, true);
        assert_eq!(result.runtime.resume, true);
        assert_eq!(result.runtime.layers, Vec::<String>::new());
    }
}
