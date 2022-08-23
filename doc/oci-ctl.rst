OCI-CTL(8)
==========

NAME
----

**oci-ctl** - Load and Register OCI applications

SYNOPSIS
--------

.. code:: bash

   USAGE:
       oci-ctl <SUBCOMMAND>

   OPTIONS:
       -h, --help       Print help information
       -V, --version    Print version information

   SUBCOMMANDS:
       build-deb    Build container package
       help         Print this message or the help of the given subcommand(s)
       list         List registered container applications
       load         Load container
       register     Register container application
       remove       Remove application registration or entire container

DESCRIPTION
-----------

oci-ctl is the control program to register and manage host applications
which actually runs inside of an OCI container. An application registered
via oci-ctl can be called on the host like a native application just
by calling the name used in the registration process.

SEE ALSO
--------

oci-pilot(8), oci-ctl-build-deb(8), oci-ctl-list(8), oci-ctl-load(8),
oci-ctl-register(8), oci-ctl-remove(8)

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
