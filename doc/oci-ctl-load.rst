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
           --oci <OCI>    OCI container to load into local podman registry
       -V, --version      Print version information


DESCRIPTION
-----------

Load the given OCI container into the local registry.
The command is based on **podman load**. After completion
the container can be listed via:

.. code:: bash

   $ podman images

OPTIONS
-------

--oci <OCI>

  OCI container to load into local podman registry. The given
  container must be in the OCI tar format like it is produced
  when exporting containers from registries via **podman export**

EXAMPLE
-------

.. code:: bash

   $ oci-ctl load --oci SOME.docker.tar

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
