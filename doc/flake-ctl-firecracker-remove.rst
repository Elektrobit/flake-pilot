FLAKE-CTL-FIRECRACKER-REMOVE(8)
===============================

NAME
----

**flake-ctl firecracker remove** - Remove application registration and/or entire VM

SYNOPSIS
--------

.. code:: bash

   USAGE:
       flake-ctl firecracker remove <--vm <VM>|--app <APP>>

   OPTIONS:
       --app <APP>
       --vm <VM>

DESCRIPTION
-----------

Remove registration(s). The command operates in two modes:

1. Remove an application registration provided via **--app**

   In this mode the command deletes the specified application if it
   is a link pointing to :file:`/usr/bin/firecracker-pilot`. It then also
   deletes the application configuration from :file:`/usr/share/flakes`

2. Remove a VM including all its registered applications via **--vm**

   In this mode the command deletes all application registrations
   using the specified VM. At the end also the specified
   VM will be removed from the local firecracker registry

OPTIONS
-------

--app <APP>

  Application absolute path to be removed from host

--vm <VM>

  VM basename as provided via **ls -1 /var/lib/firecracker/images**

FILES
-----

* /usr/share/flakes
* /var/lib/firecracker/images

EXAMPLE
-------

.. code:: bash

   $ flake-ctl firecracker remove --app /usr/bin/apt-get

   $ flake-ctl firecracker remove --vm SOME_FIRECRACKER_VM

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
