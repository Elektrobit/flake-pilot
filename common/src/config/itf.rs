use bitflags::bitflags;
use nix::unistd::User;
use std::{default::Default, path::PathBuf};

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

    pub(crate) paths: FlakeCfgPaths,
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

    /// Get `paths` namespace, represented by [`FlakeCfgPaths`] struct.
    pub fn paths(&self) -> &FlakeCfgPaths {
        &self.paths
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
            paths: FlakeCfgPaths::default(),
        }
    }
}

/// Paths for various command proxypass between the guest and host
#[derive(Debug, Clone)]
pub struct FlakeCfgPaths {
    // Path to the target on the container/VM, that needs to be exported
    pub(crate) exported_app_path: PathBuf,

    // Path on the host where flake is installed as executable symlink
    // to launch exported app
    pub(crate) registered_app_path: PathBuf,

    // Path to rootfs image done by app registration
    pub(crate) vm_rootfs_path: Option<PathBuf>,

    // Path to kernel image done by app registration
    pub(crate) vm_kernel_path: Option<PathBuf>,

    // Optional path to initrd image done by app registration
    pub(crate) vm_initrd_path: Option<PathBuf>,
}

impl FlakeCfgPaths {
    /// Returns the initrd path for [`FlakeCfgPaths`].
    /// The initrd path meant to be of a mounted image.
    /// See [`FlakeCfgPaths::vm_rootfs_path`]
    pub fn vm_initrd_path(&self) -> Option<&PathBuf> {
        self.vm_initrd_path.as_ref()
    }

    /// Returns the kernel path for [`FlakeCfgPaths`].
    /// The kernel path meant to be on a mounted image.
    /// See [`FlakeCfgPaths::vm_rootfs_path`]
    pub fn vm_kernel_path(&self) -> Option<&PathBuf> {
        self.vm_kernel_path.as_ref()
    }

    /// Returns the rootfs path for [`FlakeCfgPaths`].
    /// The rootfs is expected to be mounted.
    pub fn vm_rootfs_path(&self) -> Option<&PathBuf> {
        self.vm_rootfs_path.as_ref()
    }

    /// Returns a reference to the exported app path of [`FlakeCfgPaths`].
    /// **Exported App Path** is a path of an application inside of a flake
    /// which should be launched. It also can be wrapped in
    /// any way: scripts or binary launchers.
    pub fn exported_app_path(&self) -> &PathBuf {
        &self.exported_app_path
    }

    /// Returns a reference to the registered app path of [`FlakeCfgPaths`].
    /// **Registered App Path** is a path of a symlink on the host machine
    /// where Flake is installed. Calling this path will eventually call
    /// `exported_app_path` function.
    /// See [`FlakeCfgPaths::exported_app_path`].
    pub fn registered_app_path(&self) -> &PathBuf {
        &self.registered_app_path
    }
}

impl Default for FlakeCfgPaths {
    fn default() -> Self {
        Self {
            exported_app_path: PathBuf::new(),
            registered_app_path: PathBuf::new(),
            vm_initrd_path: None,
            vm_kernel_path: None,
            vm_rootfs_path: None,
        }
    }
}

bitflags! {
    /// InstanceMode defines instance behaviour. For example, containers
    /// can be resumed, attached or volatile (one-timers those are copied,
    /// launched and then removed). Partially this behaviour can be for
    /// virtual machines as well: attached (i.e. running) or resumed (restarted).
    #[derive(Debug, Clone)]
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

    // Boot arguments. These are used by VM type
    pub(crate) vm_boot_args: Option<Vec<String>>,
    pub(crate) vm_mem_size_mib: Option<u32>,
    pub(crate) vm_vcpu_count: Option<u8>,
    pub(crate) vm_cache_type: Option<CacheType>,

    // Size of the VM overlay
    // If specified a new ext2 overlay filesystem image of the
    // specified size will be created and attached to the VM
    pub(crate) vm_overlay_size_mib: Option<u128>,
}

impl FlakeCfgEngine {
    /// Returns the size of overlay filesystem in MiB of [`FlakeCfgEngine`].
    /// This is used for the virtual machines engine type.
    pub fn vm_overlay_size_mib(&self) -> Option<u128> {
        self.vm_overlay_size_mib
    }

    /// Returns the cache type of of [`FlakeCfgEngine`].
    /// This is used for the virtual machines engine type.
    pub fn vm_cache_type(&self) -> Option<&CacheType> {
        self.vm_cache_type.as_ref()
    }

    /// Returns could of virtual CPUs of [`FlakeCfgEngine`].
    /// This is used for the virtual machines engine type.
    pub fn vm_vcpu_count(&self) -> Option<u8> {
        self.vm_vcpu_count
    }

    /// Returns the size of memory in MiB of [`FlakeCfgEngine`].
    /// This is used for the virtual machines engine type.
    pub fn vm_mem_size_mib(&self) -> Option<u32> {
        self.vm_mem_size_mib
    }

    /// Returns the boot argumenta of [`FlakeCfgEngine`].
    /// This is used for the virtual machines engine type.
    pub fn vm_boot_args(&self) -> Option<&Vec<String>> {
        self.vm_boot_args.as_ref()
    }

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
}

impl Default for FlakeCfgEngine {
    fn default() -> Self {
        Self {
            pilot: "".to_string(),
            args: None,
            vm_boot_args: None,
            vm_mem_size_mib: None,
            vm_vcpu_count: None,
            vm_cache_type: None,
            vm_overlay_size_mib: None,
        }
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
