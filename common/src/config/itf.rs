use bitflags::bitflags;
use nix::unistd::User;
use serde_yaml::Value;
use std::{
    collections::{HashMap, hash_map::Keys},
    default::Default,
    hash::Hash,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

/// FlakeConfig is an interface for all configuration possible
/// across all suppirted versions of the config.
///
#[derive(Debug, Clone)]
pub struct FlakeConfig {
    // Major version of the configuration. If not defined, then v1.
    // Versions are using only major numbers: 1, 2, 3...
    pub(crate) version: u8,
    pub(crate) runtime: FlakeCfgRuntime,
    pub(crate) engine: FlakeCfgEngine,
    pub(crate) static_data: FlakeCfgStatic,
    pub(crate) setup: FlakeCfgSetup,
}

impl FlakeConfig {
    /// Create an instance of a FlakeConfig
    pub fn new(version: Option<u8>) -> Self {
        let mut fc = FlakeConfig::default();
        if let Some(version) = version {
            fc.version = version;
        }

        fc
    }

    /// Get config version
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Get runtime namespace
    pub fn runtime(&self) -> &FlakeCfgRuntime {
        &self.runtime
    }

    /// Get engine namespace
    pub fn engine(&self) -> &FlakeCfgEngine {
        &self.engine
    }

    /// Get setup namespace
    pub fn setup(&self) -> &FlakeCfgSetup {
        &self.setup
    }

    /// Get static data namespace
    pub fn static_data(&self) -> &FlakeCfgStatic {
        &self.static_data
    }
}

impl Default for FlakeConfig {
    fn default() -> Self {
        FlakeConfig {
            version: 1,
            runtime: FlakeCfgRuntime::default(),
            engine: FlakeCfgEngine::default(),
            setup: FlakeCfgSetup {},
            static_data: FlakeCfgStatic { bundles: None },
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PathMap {
    pub(crate) inner: HashMap<PathBuf, FlakeCfgPathProperties>,
}

impl Deref for PathMap {
    type Target = HashMap<PathBuf, FlakeCfgPathProperties>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PathMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl PathMap {
    pub fn new() -> Self {
        PathMap { inner: HashMap::new() }
    }

    /// Get properties by a PathBuf.
    pub fn get_by_path(&self, p: PathBuf) -> Option<&FlakeCfgPathProperties> {
        if let Some(p) = self.get(&p) {
            return Some(p);
        }
        self.get(&PathBuf::from(p.file_name().unwrap()))
    }
}

/// FlakeConfigRuntime is a namespace for all runtime-related
/// configuration options
#[derive(Debug, Clone)]
pub struct FlakeCfgRuntime {
    // Image name in the registry, mostly used by OCI containers.
    // Can be used by other image storages, if needed.
    pub(crate) image_name: String,

    // Runtime-agnostic layering definition
    pub(crate) base_layer: Option<String>,
    pub(crate) layers: Option<Vec<String>>,

    // Run as defined user. If None, then proxy-pass current user.
    pub(crate) run_as: Option<User>,

    pub(crate) instance_mode: InstanceMode,

    //pub(crate) paths: HashMap<PathBuf, FlakeCfgPathProperties>,
    pub(crate) paths: PathMap,
}

impl FlakeCfgRuntime {
    /// Get the image name.
    ///
    /// The image name is used mostly for the containers to be searched
    /// in their registry. Can be used by other image storages, if needed.
    pub fn image_name(&self) -> &str {
        self.image_name.as_ref()
    }

    /// Get name of a base layer, if any.
    pub fn base_layer(&self) -> Option<&String> {
        self.base_layer.as_ref()
    }

    /// Get a list of layers on top of base later, if any.
    pub fn layers(&self) -> Option<&Vec<String>> {
        self.layers.as_ref()
    }

    /// Get a [`crate::user::User`] instance, used to run
    /// as it. This is **discouraged** as it requires use of `sudo`.
    pub fn run_as(&self) -> Option<User> {
        self.run_as.to_owned()
    }

    /// Get instance mode which is described in [`InstanceMode`] enum.
    /// Default is set to [`InstanceMode::Volatile`].
    pub fn instance_mode(&self) -> &InstanceMode {
        &self.instance_mode
    }

    /// Get the path-map
    pub fn paths(&self) -> &PathMap {
        &self.paths
    }

    /// Returns a tuple containing the "proper" name of the flake and an iterator over all other paths
    /// 
    /// Returns `None` if there are not paths in the config
    pub fn get_symlinks(&self) -> Option<(&PathBuf, Keys<'_, PathBuf, FlakeCfgPathProperties>)> {
        let mut paths = self.paths.keys();
        paths.next().map(|first| (first, paths))
    }
}

impl Default for FlakeCfgRuntime {
    fn default() -> Self {
        Self {
            image_name: "".to_string(),
            base_layer: None,
            layers: None,
            run_as: None,
            instance_mode: InstanceMode::default(),
            paths: PathMap::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FlakeCfgPathProperties {
    pub(crate) exports: PathBuf,
    pub(crate) run_as: Option<User>,
    pub(crate) instance_mode: Option<InstanceMode>,
}

impl FlakeCfgPathProperties {
    pub fn new(exports: PathBuf) -> Self {
        FlakeCfgPathProperties { run_as: None, instance_mode: None, exports }
    }

    /// Returns a reference to the exports of this [`FlakeCfgPathProperties`].
    pub fn exports(&self) -> &PathBuf {
        &self.exports
    }

    /// Returns the user to run-as of this [`FlakeCfgPathProperties`].
    pub fn run_as(&self) -> Option<&User> {
        self.run_as.as_ref()
    }

    /// Returns the instance mode of this [`FlakeCfgPathProperties`].
    pub fn instance_mode(&self) -> Option<InstanceMode> {
        self.instance_mode
    }
}

bitflags! {
    /// InstanceMode defines instance behaviour. For example, containers
    /// can be resumed, attached or volatile (one-timers those are copied,
    /// launched and then removed). Partially this behaviour can be for
    /// virtual machines as well: attached (i.e. running) or resumed (restarted).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct InstanceMode: u32 {
        const Volatile = 1 << 0; // One-timer, None
        const Resume = 1 << 1;
        const Attach = 1 << 2;
    }
}

impl Default for InstanceMode {
    fn default() -> Self {
        Self::Volatile
    }
}

/// Cache type for VM
#[derive(Debug, Clone)]
pub enum CacheType {
    Writeback,
}

impl Default for CacheType {
    fn default() -> Self {
        Self::Writeback
    }
}

/// FlakeConfigEngine is a namespace for all engine-related
/// configuration options
#[derive(Debug, Clone)]
pub struct FlakeCfgEngine {
    // Name of the pilot. It sticks to the conventional name:
    //
    //    <NAME>-pilot
    //
    // In this case "name" is what goes to the "pilot" variable.
    pub(crate) pilot: String,

    // Proxy-pass static arguments "as is". Argument can be anything:
    // flags, keywords like "--foo=bar" etc. These arguments do
    // not allow to interpolate anything. For example, "--foo=$BAR"
    // will literally pass "$" as an escaped character.
    pub(crate) args: Option<Vec<String>>,

    // Arbitrary internal params, those are only known to a specific pilot
    // per its separate documentation. The type here is serde_yaml::Value
    // and the real content should be either extracted separately or
    // extra deserialised into a specific struct, those are extra for
    // the specific pilots.
    //
    // Optional.
    pub(crate) params: Option<Value>,
}

impl FlakeCfgEngine {
    /// Returns a list of runtime args for [`FlakeCfgEngine`].
    pub fn args(&self) -> Option<Vec<String>> {
        self.args.to_owned()
    }

    /// Returns the name of the pilot for [`FlakeCfgEngine`].
    /// Pilot name is taken from the list existing available pilots in the system.
    /// A pilot naming convention is `<NAME>-pilot`, where `name` is returned
    /// by this method without its suffix.
    pub fn pilot(&self) -> &str {
        self.pilot.as_ref()
    }

    pub fn params(&self) -> Option<Value> {
        self.params.clone()
    }
}

impl Default for FlakeCfgEngine {
    fn default() -> Self {
        Self { pilot: "".to_string(), args: None, params: None }
    }
}

/// FlakeConfigSetup is a namespace for all configuration options
/// related to the Flake setup, such as permission access to
/// the media, X11, directories etc
#[derive(Debug, Clone)]
pub struct FlakeCfgSetup {}

/// Static data.
/// It is all kind of stuff that will be written over the rootfs
/// on specific mountpoint of the *instance* (not on the source image!).
///
/// Static data can be added only as archives
/// and they should resemble the tree starting from
/// the root ("/"). If it is a package, it will be
/// extracted to the rootfs from that mountpoint,
/// like it would be installed, except its scriptlets
/// won't be launched.
#[derive(Debug, Clone)]
pub struct FlakeCfgStatic {
    pub(crate) bundles: Option<Vec<String>>,
}

impl FlakeCfgStatic {
    /// Get a list of archives (bundles) those are located in
    /// the configuration area or any other pilot-specific places.
    pub fn get_bundles(&self) -> Option<&[String]> {
        self.bundles.as_deref()
    }
}
