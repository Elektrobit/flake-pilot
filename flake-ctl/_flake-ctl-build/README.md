Tool for creating installation packages (.deb, .rpm, etc.) from flakes. The packages contain all needed data and dependencies (pilots, etc.) to run the flake on linux.

When running `flake-ctl build` the native package manager for your system will be chosen automatically. However you can run run packaging for different package managers manually by running `flake-ctl build-<manager>`.

Currently flakes can be packaged in two modes:
- `flake` will package an exisitng flake as-is
- `image` will generate a new flake from the given image, package it, then discard it

## Ecosystem
The capability to export a flake in a format that can be packaged is provided by `flake-ctl-<engine>` via the `export` command. The capability to package an exported flake into an actual package is provided by `flake-ctl-build-<manager>`


Currently supported out of the box:

**Exportable Engines**
- `flake-ctl-podman` (oci containers)

**Packagers**
- `flake-ctl-build-dpkg` (.deb for debian, ubuntu, etc.)
- `flake-ctl-build-rpmbuild` (.rpm for redhat, etc.)

## Configuration
### Package
Each package needs the following fields
```
name
description
version
url
maintainer_name
maintainer_email
license
```
They can be provided as command line argumments (`--name bla`), as environment variables (`PKG_FLAKE_NAME=bla`), or as fields in `options.yaml` which can be loacted in `./.flakes/package/options.yaml` or `~/.flakes/package/options.yaml`. The order of precedence for each field is (increasing):

- global `options.yaml`
- local `options.yaml`
- command line

Alls fields that are set in neither will be taken from environment variables.

If a field is still not set after that the user will be asked to input the field on the command line. If it is still blank, the build process wil be abborted.

### Flake
All arguments given to a builder after a double dash will be forwarded to the corresponding `flake-ctl-<engine> register` command, this way flakes can be modified before packaging. This option currently has no effect if packaging in `flake` mode.

See `flake-ctl-<engine> help` to see which arguments are valid for an engine.