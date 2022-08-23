OCI-CTL-REGISTER(8)
===================

NAME
----

**oci-ctl register** - Register container application

SYNOPSIS
--------

.. code:: bash

   USAGE:
       oci-ctl register [OPTIONS] --container <CONTAINER> --app <APP>

   OPTIONS:
       --app <APP>                An absolute path to the application inside the container. If not
                                  specified via the target option, the application will be
                                  registered with that path on the host
       --container <CONTAINER>    A container name. The name must match with a name in the local
                                  podman registry
   -h, --help                     Print help information
       --target <TARGET>          An absolute path to the application on the host. Use this option
                                  if the application path on the host should be different to the
                                  application path inside of the container
   -V, --version                  Print version information

DESCRIPTION
-----------

Register the given application to run inside of the specified container.
The registration process is two fold:

1. Create the application symlink pointing to :file:`/usr/bin/oci-pilot`
2. Create the application default configuration below :file:`/usr/share/flakes`.
   Each application registered is called a **flake**

On successful completion the registered *--app* name can be called
like a normal application on this host.

For further details about the flake configuration please refer to
the **oci-pilot** manual page.

OPTIONS
-------

--app <APP>

  An absolute path to the application inside the container. If not
  specified via the target option, the application will be
  registered with that path on the host

--container <CONTAINER>

  A container name. The name must match with a name in the local
  podman registry

--target <TARGET>

  An absolute path to the application on the host. Use this option
  if the application path on the host should be different to the
  application path inside of the container

FILES
-----

* /usr/share/flakes

EXAMPLE
-------

.. code:: bash

   $ oci-ctl register --container SOME_APT_CONTAINER \
       --app /usr/bin/apt-get

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
