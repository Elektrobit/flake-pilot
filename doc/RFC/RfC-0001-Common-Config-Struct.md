# Common Config Sruct

| Status        | Proposed                                             |
:-------------- |:---------------------------------------------------- |
| **RFC #**     | (update when you have a PR #)                        |
| **Author(s)** | Michael Meyer (ichmed95@gmail.com)                   |
| **Sponsor**   |                           |
| **Updated**   | 2023-09-12                                           |

## Objective

Consolidate the representation of the flake yaml configs into a single Rust struct and future proof their layout.

Goals:
- Single struct that can represent all pilot configs
- Validate config once and provide type safe acces afterwards
- Keep structure human readable and accessible to pilots that do not use the Rust crate
- Provide a uniform Header section that contains general information to be used by internal and external tooling
- Retain the ability to merge multiple config files before validation

Non-Goals:
- Provide implementation for specific pilot configs

## Motivation

Right now there are implementations of a config struct in:
- podman-pilot
- firecracker-pilot
- flake-ctl

Theses are highly overlapping and in case of flake-ctl direct doubles. This increase the maintenance effort needlessly.

Right now the configs are also missing information about what engine they are meant for, making debugging harder.

## User Benefit

None directly, external behaviour should remain unchanged.
Lays the groundwork to improve user facing tooling in the future.

## Design Proposal

Provide a generic config struct in the form of
```rust
pub struct Config<R> {
    /// The unique name of this flake
    pub name: String,
    /// Location of the coresponding pilot link
    pub host_path: PathBuf,
    pub include: Vec<String>,
    // Engine section
    /// What engine-type is this flake meant for
    pub engine_type: Option<String>,
    /// The runtime specific options for this engine
    pub runtime: R,
}
```
Where `R` can be any deserializable struct that implements the `EngineConfig` trait.
```rust
pub trait EngineConfig {
    /// The type of engine, e.g. "container" or "vm"
    fn engine_type() -> Option<String>;
    /// The specific engine to be used, e.g. "podman", "runc", "firecracker"
    fn engine() -> Option<String>;
}
```
There will be a special version `Config<serde_yaml::Value>` that can hold information about _any_ flake but does not allow for easy accessing of the values. 

The general structure of the yamls will look like this

```yaml
name: name_of_this_flake
host_path: path/to/the/app
include: []
engine_type: <type> # e.g. container, vm, wasm, ...
    runtime:
        <type>:
            # specific values for all engines of this type
        engine_1:
            # specific values used only by engine_1
        engine_2:
            # specific values used only by engine_2

```

Requesting a `Config<Engine>` will look up the `engine_type` and name of `Engine` via the `EngineConfig` trait and extract the fields for both from the yaml. The values of the type-section and the engine-specific section will be merged with the specific section taking precendence.

As long as permitted by `Engine`s layout, there need not be an engine_specific section for certain engines.

There will be a special `PartialConfig` struct that represents a maybe-invalid configuration that can be combined with other `PartialConfig`s to produce a valid configuration. This is used to combine multiple yaml files into one config.

### Alternatives Considered

- Ad-hoc access into a generic serde_yaml::Mapping
- Individual Config struct for each pilot

### Performance Implications

None

### Dependencies

Removes the dependency on `yaml-rust`

### Engineering Impact
The `Config<T>` struct will be maintained int the `common` crate. All implementations of `T` will be maintained in the coresponding pilot code.

`Config<T>` can be tested independendly.
`Config<T>` is exported publicly and can be use for internal and external development of tools and pilots.

### Compatibility

Exisitng config files will have to be migrated.

## Detailed Design

## Questions and Discussion Topics

- Should the field for engint-type setting be called "type" or "_type"?
- Are there fields besides name, host_app_path and includes that are universal to all potential flake engines?
