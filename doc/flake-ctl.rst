FLAKE-CTL(8)
============

NAME
----

**flake-ctl** - Load and Register flake applications

SYNOPSIS
--------

.. code:: bash

   USAGE:
       flake-ctl <SUBCOMMAND>

   OPTIONS:
       -h, --help       Print help information
       -V, --version    Print version information

   SUBCOMMANDS:
       help         Print this message or the help of the given subcommand(s)
       list         List registered container applications
       podman       Load and register OCI applications
       firecracker  Load and register VM applications

DESCRIPTION
-----------

flake-ctl is the control program to register and manage flake applications
which actually runs inside of an instance created by a runtime engine.
Currently supported runtime engines are:

* podman
* firecracker

An application registered via flake-ctl can be called on the host like a
native application just by calling the name used in the
registration process.

SEE ALSO
--------

podman-pilot(8), flake-ctl-podman-build-deb(8), flake-ctl-list(8), flake-ctl-podman-load(8), flake-ctl-podman-register(8), flake-ctl-podman-remove(8), firecracker-pilot(8), flake-ctl-firecracker-load(8), flake-ctl-firecracker-register(8), flake-ctl-firecracker-remove(8)

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
