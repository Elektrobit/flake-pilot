OCI-CTL-REMOVE(8)
=================

NAME
----

**oci-ctl remove** - Remove application registration and/or entire container

SYNOPSIS
--------

.. code:: bash

   USAGE:
       oci-ctl remove <--container <CONTAINER>|--app <APP>>

   OPTIONS:
       --app <APP>                Application absolute path to be removed from host
       --container <CONTAINER>    Remove all applications registered with the given container
                                  and also remove the container from the local podman registry
   -h, --help                     Print help information
   -V, --version                  Print version information

DESCRIPTION
-----------

Remove registration(s). The command operates in two modes:

1. Remove an application registration provided via **--app**

   In this mode the command deletes the specified application if it
   is a link pointing to :file:`/usr/bin/oci-pilot`. It then also
   deletes the application configuration from :file:`/usr/share/flakes`

2. Remove a container including all its registered applications via **--container**

   In this mode the command deletes all application registrations
   using the specified container. At the end also the specified
   container will be removed from the local podman registry
   
OPTIONS
-------

--app <APP>

  Application absolute path to be removed from host

--container <CONTAINER>

  Container basename as provided via **podman images**

FILES
-----

* /usr/share/flakes

EXAMPLE
-------

.. code:: bash

   $ oci-ctl remove --app /usr/bin/apt-get

   $ oci-ctl remove --container SOME_APT_CONTAINER

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
