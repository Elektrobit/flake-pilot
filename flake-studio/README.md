`flake-studio` provides a project based iterrative environment for packaging flakes, where you can create a package, inspect it, modify your settings and build it again.

It is also possible to check these projects into version control.

# Setup
Run `flake-studio new my_app` to create a new project in your current directory or run `flake-studio init` to turn your current directory _into_ a project.

For now you have to fill out all fields in the wizard. If you wan to pre-fill some of theses fields put them into `~/.flakes/package/options.yaml`.

Update `build.sh` in the `src/` directory of your new project

# Building
To build your package simply run `flake-studio build` from anywhere inside your project. The final package will be contained in `out/`.

Right now `flake-studio` will always use the package format native to your linux distribution. If you wan to use another package manager use `flake-ctl-build` instead.

# Contents of `src/`
Listed bellow are all files and directories with special meanings. Any other contents of the `src/` directory will be ignored by `flake-studio`.

## Expected
### options.yaml
Includes the settings for your package see the `flake-ctl-build` documentation for more information.

### flake.yaml
Contains the base configuration of your flake, this file will automatically be included with every build as `<my_app>.yaml`.

### build.sh
This script gets called with the name of an image as its only parameter and should create or provide an image with the same name in the appropriate place (for podman this would be the local image registry).

There are multiple ways to do this, the auto-generated build.sh includes code to run a simple `Dockerfile`.

If you want to run any other programs or modify your image before packaging, this is also the place to do it.

### out/
This directory will be created during the build if it does not exist and contains the final packages.

### .staging/
This directory is created by `flake-studio` and contains the structure needed to run `flake-ctl-build compile`.

You should not manually modify this directory.

### .flakes/package
This directory contains metadata like `options.yaml`.

When using a `flake-studio` project you should not modify the contents directly as they may get overwritten.

### .flakes/studio
Empty directory (for now) that marks the containing directory as a `flake-studio` project.

## Optional
### flake.d
This directory is not automatically created but if you add it yourself the contentes will be included as overrides in `<my_app>.d`, the internal files will not be renamed.
