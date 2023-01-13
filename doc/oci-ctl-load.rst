OCI-CTL-LOAD(8)
===============

NAME
----

**oci-ctl load** - Load container to local registry

SYNOPSIS
--------

.. code:: bash

   USAGE:
       oci-ctl load --oci <OCI>

   OPTIONS:
       -h, --help         Print help information
           --oci <OCI>    OCI image to load into local podman registry
           --remove       Remove given OCI image after loading to local registry
       -V, --version      Print version information


DESCRIPTION
-----------

Load the given OCI image into the local registry.
The command is based on **podman load**. After completion
the container can be listed via:

.. code:: bash

   $ podman images

OPTIONS
-------

--oci <OCI>

  OCI image to load into local podman registry. The given
  container must be in the OCI tar format like it is produced
  when exporting containers from registries via **podman export**

--remove

  Remove given OCI image file, after successful loading into the
  local registry.

EXAMPLE
-------

.. code:: bash

   $ oci-ctl load --oci SOME.docker.tar --remove

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
