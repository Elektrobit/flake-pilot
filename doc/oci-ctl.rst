OCI-CTL(8)
==========

NAME
----

**oci-ctl** - Load and Register flake applications

SYNOPSIS
--------

.. code:: bash

   USAGE:
       oci-ctl <SUBCOMMAND>

   OPTIONS:
       -h, --help       Print help information
       -V, --version    Print version information

   SUBCOMMANDS:
       help         Print this message or the help of the given subcommand(s)
       list         List registered container applications
       podman       Load and register OCI applications

DESCRIPTION
-----------

oci-ctl is the control program to register and manage flake applications
which actually runs inside of an instance created by a runtime engine.
Currently supported runtime engines are:

* podman

An application registered via oci-ctl can be called on the host like a
native application just by calling the name used in the
registration process.

SEE ALSO
--------

oci-pilot(8), oci-ctl-podman-build-deb(8), oci-ctl-list(8), oci-ctl-podman-load(8),
oci-ctl-podman-register(8), oci-ctl-podman-remove(8)

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
