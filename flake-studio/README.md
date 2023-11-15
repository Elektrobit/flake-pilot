# Setup
Run `flake-studio init /path/to/app` to turn your current directory into a flake-studio project

For now you have to fill out all fields in the wizard.

Put a file called `build.sh` into `src/`, mark its as executable and put `#! /bin/sh` inside

# Building
Modify the files in src

Run `flake-studio build`

The resulting package should be located in `out/`