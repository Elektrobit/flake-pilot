FLAKE-CTL-BUILD(8)
===============================

NAME
----

**flake-ctl build** - Package existing flakes or images with the native package manager

SYNOPSIS
--------

.. code:: bash

    Usage: flake-ctl-build [OPTIONS] -- [TRAILING..]

    Options:
    -a, --app <APP>
            Name of the app on the host
    -c, --from-oci <OCI>
            Build from the given oci container
    -t, --from-tar <TAR>
            Build from tarball. NOTE: file should have extension ".tar.gz"
    -o, --output <OUTPUT>
            Output directory
    -l, --location <LOCATION>
            Where to build the package (default: tempdir)
    
    Trailing:
    Trailing arguments are forwarded to "flake-ctl <pilot> register"

DESCRIPTION
-----------
Create a software package containing a flake using the system native package manager (e.g. dpkg on ubuntu or rpm on redhat)

The resulting package will contain only the files needed to run the flake. The package will have a dependency on a flake 
pilot (e.g. flake-pilot-podman for podman), which in turn may have further dependencies.

Passing options after the '--' will forward these options to `flake-ctl <pilot> register`, allowing you to customize the setup of the flake.

OPTIONS
-------
-a, --app <APP>
        Name of the app on the host
-c, --from-oci <OCI>
        Build from the given oci container
-t, --from-tar <TAR>
        Build from tarball. NOTE: file should have extension ".tar.gz"
-o, --output <OUTPUT>
        Output directory
-l, --location <LOCATION>
        Where to build the package (default: tempdir)
    --dry-run
        Only assemble bundle, do not compile
    --compile
        Compile a pre-existing bundle given by 'location'
    --no-image
        Do not include an image in this flake
    --ci
        Skip all potential user input
    --image <IMAGE>
        Override the image used in the flake (otherwise inferred)
    --name <NAME>
        The name of the package (excluding version, arch, etc.)
    --description <DESCRIPTION>
        
    --version <VERSION>
        
    --url <URL>
        A url pointing to the packages source
    --maintainer-name <MAINTAINER_NAME>
        
    --maintainer-email <MAINTAINER_EMAIL>
        
    --license <LICENSE>
        
    --template <TEMPLATE>
        Location of .spec template and pilot specific data [default: /usr/share/flakes/package/dpkg]
-h, --help
        Print help

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
- for the current working directory in `./.flakes/package/options.yaml`
- over the cli as options

If an option was not specified in any of the ways mentioned above the user will be prompted for a value when not running in CI-mode.
When running in CI-mode the program will instead terminate if not all options were given.

EXAMPLE
-------

.. code:: bash

   $ flake-ctl build --from-oci=my_image --app=/usr/bin/myapp

   $ flake-ctl build --from-oci=ubuntu --app=/usr/bin/flashbake --version 0.0.1 --name flashbake -- --target /usr/bin/bash
