# Flakes Configuration (v2)

This is the specs of the configuration v2 for Flakes.
Configuration for Flakes is always forward-compatible.

## What's New

Configuration difference between v1 and v2:

- Allows export of multiple commands
- Allows different behaviours of the same command
- Discourages hard-coding to a specific pilot
- Extendable

```yaml
# Version of the configuration.
# Default: 1
version: 2

# All settings for all engines during their runtime
runtime:
  name: "name in local registry"

  # Map of the exported path.
  # Config v1 allows only one app per a flake. It does not
  # allow to export multiple commands and does not allow to have
  # different behaviours per a command.
  #
  # Config v2 solving this problem by having export path map
  # where internal flake command exports to an external command
  # on the host machine.
  #
  # Mandatory, should have at least one path defined
  path_map:
    # Command that will appear on host machine
    /usr/bin/banana:

      # Exported path from flake. Default is the same as host path.
      #
      # Optional
      exports: /usr/bin/foo

      # Override general "user" option, specified below
      #
      # Optional
      user: root

    # Command that will appear on host machine
    /usr/bin/rotten-banana:

      # Exported path from flake (same as above, but does not requires root,
      # however adds resume flag)
      exports: /usr/bin/foo

      # override general "instance" option, specified below
      instance: resume

    # Another flake command "just-like-that"
    /usr/bin/just-like-that:
      # ...is exported as "/usr/bin/bar"
      exports: /usr/bin/bar

    # Empty command will be exported to the same path on host machine
    /usr/bin/bash:

  # Layering
  base_layer:
  layers:
    - one
    - two

  # General runtime behaviour (overridable in "host_app_paths")
  # If not specified, then it is a current user
  user:

  # Instance behaviour flags. Separated by a space
  # Flags: resume, attach
  instance: resume attach

# Engine settings (per pilot)
engine:
  pilot: podman
  args:
    - -x
    - --foo=bar

# Stuff that will be written over the rootfs
# on specific mountpoint. Can be only archives
# and they should resemble the tree starting from
# the root ("/"). If it is a package, it will be
# extracted to the rootfs from that mountpoint,
# like it would be installed, except its scriptlets
# won't be launched.
static:
  - some-configs.tar.gz
  - extra-files.tar.xz
  - debian-package.deb
  - redhat-package.rpm
```
