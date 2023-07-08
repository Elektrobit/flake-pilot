FLAKE-CTL-PODMAN-REMOVE(8)
==========================

NAME
----

**flake-ctl podman remove** - Remove application registration and/or entire container

SYNOPSIS
--------

.. code:: bash

   USAGE:
       flake-ctl podman remove <--container <CONTAINER>|--app <APP>>

   OPTIONS:
       --app <APP>
       --container <CONTAINER>

DESCRIPTION
-----------

Remove registration(s). The command operates in two modes:

1. Remove an application registration provided via **--app**

   In this mode the command deletes the specified application if it
   is a link pointing to `/usr/bin/podman-pilot`. It then also
   deletes the application configuration from `/usr/share/flakes`

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

   $ flake-ctl podman remove --app /usr/bin/apt-get

   $ flake-ctl podman remove --container SOME_APT_CONTAINER

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
