OCI-CTL-BUILD-DEB(8)
====================

NAME
----

**oci-ctl build-deb** - Build debian package from OCI container

SYNOPSIS
--------

.. code:: bash

   USAGE:
       oci-ctl build-deb [OPTIONS] --oci <OCI> --repo <REPO>

   OPTIONS:
        --app <APP>...    An absolute path to the application for registration at install time of
                          the package
    -h, --help            Print help information
        --oci <OCI>       OCI container to load into local podman registry
        --repo <REPO>     Output directory to store package(s) as local debian repository
    -V, --version         Print version information

DESCRIPTION
-----------

The build-deb command takes an OCI tar container and packages it into a debian (.deb)
package. The produced package will be placed into a local debian repository such
that tools like **apt** can consume it. If provided via the **--app** option, the
package provides post install and removal scripts which registers/removes the
application for the container at install/uninstall time of the package.

OPTIONS
-------

--app <APP>...

  An absolute path to the application for registration at install
  time of the package

--oci <OCI>

  OCI container to load into local podman registry

--repo <REPO>

  Output directory to store package(s) as local debian repository

FILES
-----

* /usr/share/oci-pilot/container.spec.in
* /usr/bin/oci-deb

EXAMPLE
-------

.. code:: bash

   $ oci-ctl build-deb --oci SOME.docker.tar \
       --repo ${HOME}/localrepo \
       --app /usr/bin/myapp

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
