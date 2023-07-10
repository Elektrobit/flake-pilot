FLAKE-CTL-FIRECRACKER-REGISTER(8)
=================================

NAME
----

**flake-ctl firecracker register** - Register VM application

SYNOPSIS
--------

.. code:: bash

   USAGE:
       flake-ctl firecracker register [OPTIONS] --vm <VM> --app <APP>


   OPTIONS:
       --app <APP>
       --include-tar <INCLUDE_TAR>...
       --no-net
       --resume
       --overlay-size <OVERLAY_SIZE>
       --run-as <RUN_AS>
       --target <TARGET>
       --vm <VM>

DESCRIPTION
-----------

Register the given application to run inside of the specified firecracker
virtual machine. The registration process is two fold:

1. Create the application symlink pointing to `/usr/bin/firecracker-pilot`
2. Create the application default configuration below `/usr/share/flakes`.
   Each application registered is called a **flake**

On successful completion the registered *--app* name can be called
like a normal application on this host.

For further details about the flake configuration please refer to
the **firecracker-pilot** manual page.

OPTIONS
-------

--app <APP>

  An absolute path to the application on the host. If not specified via
  the target option, the application will be called with that path inside
  of the VM

--include-tar <INCLUDE_TAR>...

  Name of a tar file to be included on top of the VM instance.
  This option can be specified multiple times

--no-net

  Disable networking

--overlay-size <OVERLAY_SIZE>

  Size of overlay write space in bytes. Optional suffixes are:
  KiB/MiB/GiB/TiB (1024) or KB/MB/GB/TB (1000)

--resume

  Resume the VM from previous execution. If the VM is still running,
  the app will be executed inside of this VM instance

--run-as <RUN_AS>

  Name of the user to run firecracker

--target <TARGET>

  An absolute path to the application in the VM. Use this option if the application path
  on the host should be different to the application path inside of the VM

--vm <VM>

  A virtual machine name. The name must match with a name in the local firecracker
  registry


FILES
-----

* /usr/share/flakes
* /etc/flakes

EXAMPLE
-------

.. code:: bash

   $ flake-ctl firecracker register --vm NAME \
       --overlay-size 20g \
       --app /usr/bin/apt-get

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
