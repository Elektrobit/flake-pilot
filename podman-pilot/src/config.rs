use flakes::{user::User, cwd::MountMode};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{env, path::PathBuf, fs};

use crate::defaults;

lazy_static! {
    static ref CONFIG: Config<'static> = load_config();
}

/// Returns the config singleton
/// 
/// Will initialize the config on first call and return the cached version afterwards
pub fn config() -> &'static Config<'static> {
    &CONFIG
}

fn get_base_path() -> PathBuf {
    which::which(env::args().next().expect("Arg 0 must be present")).expect("Symlink should exist")
}

fn load_config() -> Config<'static> {
    /*!
    Read container runtime configuration for given program

    CONTAINER_FLAKE_DIR/
       ├── program_name.d
       │   └── other.yaml
       └── program_name.yaml

    Config files below program_name.d are read in alpha sort order
    and attached to the master program_name.yaml file. The result
    is send to the Yaml parser
    !*/
    let base_path = get_base_path();
    let base_path  = base_path.file_name().unwrap().to_str().unwrap();
    let base_yaml = fs::read_to_string(config_file(base_path));

    let mut extra_yamls: Vec<_> = fs::read_dir(config_dir(base_path))
        .into_iter()
        .flatten()
        .flatten()
        .map(|x| x.path()).collect();

    extra_yamls.sort();
        

    let full_yaml: String = base_yaml.into_iter().chain(extra_yamls.into_iter().flat_map(fs::read_to_string)).collect();
    config_from_str(&full_yaml)

}

fn config_from_str(input: &str) -> Config<'static> {
    // Parse into a generic YAML to remove duplicate keys

    let yaml = yaml_rust::YamlLoader::load_from_str(input).unwrap();
    let yaml = yaml.first().unwrap();
    let mut buffer = String::new();
    yaml_rust::YamlEmitter::new(&mut buffer).dump(yaml).unwrap();

    // Convert to a String and leak it to make it static
    // Can not use serde_yaml::from_value because of lifetime limitations
    // Safety: This does not cause a reocurring memory leak since `load_config` is only called once
    let content = Box::leak(buffer.into_boxed_str());
    
    serde_yaml::from_str(content).unwrap()
}

fn config_file(program: &str) -> String {
    format!("{}/{}.yaml", defaults::CONTAINER_FLAKE_DIR, program)
}

fn config_dir(program: &str) -> String {
    format!("{}/{}.d", defaults::CONTAINER_FLAKE_DIR, program)
}

#[derive(Deserialize)]
pub struct Config<'a> {
    #[serde(borrow)]
    pub container: ContainerSection<'a>,
    #[serde(borrow)]
    pub include: IncludeSection<'a>
}

impl<'a> Config<'a> {
    pub fn is_delta_container(&self) -> bool {
        self.container.base_container.is_some()
    }

    pub fn runtime(&self) -> RuntimeSection {
        self.container.runtime.as_ref().cloned().unwrap_or_default()
    }

    pub fn layers(&self) -> Vec<&'a str> {
        self.container.layers.as_ref().cloned().unwrap_or_default()
    }

    pub fn tars(&self) -> Vec<&'a str> {
        self.include.tar.as_ref().cloned().unwrap_or_default()
    }

    pub fn mount(&self) -> Option<&'a str> {
        self.container.dir
    }
}

#[derive(Deserialize)]
pub struct IncludeSection<'a> {
    #[serde(borrow)]
    tar: Option<Vec<&'a str>>
}

#[derive(Deserialize)]
pub struct ContainerSection<'a> {
    /// Mandatory registration setup
    /// Name of the container in the local registry
    pub name: &'a str,

    /// Path of the program to call inside of the container (target)
    pub target_app_path: Option<&'a str>,

    /// Path of the program to register on the host
    pub host_app_path: &'a str,

    /// Which directory to mount as the cwd inside the container.
    /// 
    /// No directory is mounted if `dir` is not given.
    /// 
    /// Otherwise the directory from the host will be mounted into the container and the
    /// the target app will be run with that directory as the cwd.
    pub dir: Option<&'a str>,

    /// How to mount the working directory (ignored if `dir` is not given)
    #[serde(default)]
    pub dir_mount: MountMode,

    /// Optional base container to use with a delta 'container: name'
    ///
    /// If specified the given 'container: name' is expected to be
    /// an overlay for the specified base_container. podman-pilot
    /// combines the 'container: name' with the base_container into
    /// one overlay and starts the result as a container instance
    ///
    /// Default: not_specified
    pub base_container: Option<&'a str>,

    /// Optional additional container layers on top of the
    /// specified base container
    #[serde(default)]
    layers: Option<Vec<&'a str>>,

    /// Optional registration setup
    /// Container runtime parameters
    #[serde(default)]
    pub runtime: Option<RuntimeSection<'a>>,
}

#[derive(Deserialize, Default, Clone)]
pub struct RuntimeSection<'a> {
    /// Run the container engine as a user other than the
    /// default target user root. The user may be either
    /// a user name or a numeric user-ID (UID) prefixed
    /// with the ‘#’ character (e.g. #0 for UID 0). The call
    /// of the container engine is performed by sudo.
    /// The behavior of sudo can be controlled via the
    /// file /etc/sudoers
    #[serde(borrow, flatten)]
    pub runas: User<'a>,

    /// Resume the container from previous execution.
    ///
    /// If the container is still running, the app will be
    /// executed inside of this container instance.
    ///
    /// Default: false
    #[serde(default)]
    pub resume: bool,

    /// Attach to the container if still running, rather than
    /// executing the app again. Only makes sense for interactive
    /// sessions like a shell running as app in the container.
    ///
    /// Default: false
    #[serde(default)]
    pub attach: bool,

    /// Caller arguments for the podman engine in the format:
    /// - PODMAN_OPTION_NAME_AND_OPTIONAL_VALUE
    ///
    /// For details on podman options please consult the
    /// podman documentation.
    #[serde(default)]
    pub podman: Option<Vec<&'a str>>,
}

#[cfg(test)]
mod test {
    use crate::config::config_file;

    use super::config_from_str;

    #[test]
    fn simple_config() {
        let cfg = config_from_str(
r#"container:
 name: JoJo
 host_app_path: /myapp
include:
 tar: ~
"#);
        assert_eq!(cfg.container.name, "JoJo");
    }
    
    #[test]
    fn combine_configs() {
        let cfg = config_from_str(
r#"container:
 name: JoJo
 host_app_path: /myapp
include:
 tar: ~
container:
 name: Dio
 host_app_path: /other
"#);
        assert_eq!(cfg.container.name, "Dio");
    }

    #[test]
    fn test_program_config_file() {
        let config_file = config_file(&"app".to_string());
        assert_eq!("/usr/share/flakes/app.yaml", config_file);
    }
}
