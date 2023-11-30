`flake-ctl` consists of a main program with built in commands as well as several extensions (similar to git). 

Each extension contains their own README.md

The main flake-ctl currently provides two commands:
- `list` Lists all currently installed flakes
- `help` Prints the help for `flake-ctl` and all installed extensions

## Extensions
To create an extension simply have a executable called `flake-ctl-<something>` in your PATH. 

When you call `flake-ctl <something> a b c` the parameters `a`, `b` and `c` will be forwarded to your extension which will be called as `flake-ctl-<something> a b c`.

Extensions can provide an information tuple in the form of `<Description>;<TYPE>` whne they are called as `flake-ctl-<something> about`. 

For example `flake-ctl-podman about` will respond with 

```
Manage podman/oci based flakes;ENGINE
```

When calling `flake-ctl help` the description will be used as help text for your extension and the type will be used to group extensions together.

Currently there are the following "official" types, however extensions may provide any type they wish.
- `ENGINE` Creates/Updates/Removes flakes for a specific flake engine (e.g. podman, firecracker)
- `PACKAGER` Used by `flake-ctl-build` to create packages out of a flake. These extensions should be called `flake-ctl-build-<something>` in order to work correctly
- `TOOL` Generic category for tooling