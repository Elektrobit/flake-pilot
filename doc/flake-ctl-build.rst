FLAKE-CTL-BUILD(8)
===============================

NAME
----

**flake-ctl build** - Package existing flakes or images with the native package manager

SYNOPSIS
--------

.. code:: bash

    Usage: flake-ctl-build-dpkg [OPTIONS] <COMMAND> [COMMAND_ARGS] [PACKAGE_OPTIONS] -- [TRAILING..]

    Commands:
    flake  Package an existing flake
        flake_name  Name of the pre-existing flake that should be packaged
    image  Package an existing image as a flake
        pilot       The type of pilot to use for the flake
        image_name  The name of the pre-existing image to package (syntax depends on pilot)

DESCRIPTION
-----------
Create a software package containing a flake using the system native package managager (e.g. dpkg on ubuntu or rpm on redhat)

The resulting package will contain only the files needed to run the flake. The package will have a dependency on a flake 
pilot (e.g. flake-pilot-podman for podman), which in turn may have further dependencies.

When packaging an exisitng flake the flake will be kept as-is. When packaging an image, a temporary flake will be created.
Passing options after the '--' will forward these options to `flake-ctl <pilot> register`, allowing you to customize the setup of the flake.

OPTIONS
-------

--target <TARGET>    Name of the final package

--dry-run    Do not actually build the package, just prepare the bundle
    May be used for testing or to create a bundle for a build system such as obs

--keep    Keep the bundle after a dry run or packaging

--location <LOCATION>    Where to build the package;
    defaults to a tmp directory that is deleted afterwards

--ci    Run in CI (non interactive) mode

PACKAGE_OPTIONS
---------------
The following options specify meta data about the created package. Not all package managers make use of all of these fields

--name <NAME>    The name of the package (excluding version, arch, etc.)
--description <DESCRIPTION>     A short descritption of the package
--version <VERSION>     Semantic version number of the package
--url <URL>    A url pointing to the packages source
--maintainer-name <MAINTAINER_NAME>     Name of the flake's maintainer
--maintainer-email <MAINTAINER_EMAIL>   Email adresse of the flake's maintainer
--license <LICENSE>     Short name of the license (e.g. MIT)

All of these options may be specified as follows (in rising order of precedence)
- over environment variables as `PKG_FLAKE_<NAME_IN_LARGE_SNAKE_CASE>`
- globally in `~/.flakes/package/options.yaml`
- for the current working directory in `.flakes/package/options.yaml`
- over the cli as options

If an option was not specified in any of the ways mentioned above the user will be prompted for a value when not running in CI-mode.
When running in CI-mode the program will instead teminate if not all options were given.

EXAMPLE
-------

.. code:: bash

   $ flake-ctl build image podman my_image /usr/bin/myapp

   $ flake-ctl build flake my_flake

   $ flake-ctl build image podman ubuntu /usr/bin/flashbake --version 0.0.1 --name flashbake -- --target /usr/bin/bash

AUTHOR
------

Michael Meyer

COPYRIGHT
---------

(c) 2023, Elektrobit Automotive GmbH
