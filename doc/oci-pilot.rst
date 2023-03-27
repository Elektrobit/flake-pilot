OCI-PILOT(8)
============

NAME
----

**oci-pilot** - Launcher for flake applications

DESCRIPTION
-----------

A flake application is an application which gets called through
a runtime engine. As of today oci-pilot supports OCI containers
called through the podman container engine. Future releases
are planned to support other engines as well, for example
FireCracker

oci-pilot provides the application launcher binary and is not expected
to be called by users. Instead it is being used as the symlink target
at the time an application is registered via **oci-ctl register**.

This means oci-pilot is the actual binary called with any application
registration. If the registered application is requested as
:file:`/usr/bin/myapp` there will be a symlink pointing to:

.. code:: bash

   /usr/bin/myapp -> /usr/bin/oci-pilot

Consequently calling **myapp** will effectively call **oci-pilot**.
oci-pilot now reads the calling program basename, which is **myapp**
and looks up all the registration metadata stored in
:file:`/usr/share/flakes`

Below :file:`/usr/share/flakes` each application is registered
with the following layout:

.. code:: bash

   /usr/share/flakes/
       ├── myapp.d
       │   └── other.yaml
       └── myapp.yaml

All metadata information read by **oci-pilot** uses the YAML
markup. The main configuration :file:`myapp.yaml` is read first
and can be optionally extended with further :file:`*.yaml` files
below the :file:`myapp.d` directory. All files in the
:file:`myapp.d` directory will be read in alpha sort order.
Redundant information will always overwrite the former one.
Thus the last setting in the sequence wins.

From a content perspective the following registration parameters
can be set for the supported container engine:

.. code:: yaml

   container:
     # Mandatory registration setup
     # Name of the container in the local registry
     name: name

     # Path of the program to call inside of the container (target)
     target_app_path: path/to/program/in/container

     # Path of the program to register on the host
     host_app_path: path/to/program/on/host

     # Optional base container to use with a delta 'container: name'
     # If specified the given 'container: name' is expected to be
     # an overlay for the specified base_container. oci-pilot
     # combines the 'container: name' with the base_container into
     # one overlay and starts the result as a container instance
     #
     # Default: not_specified
     base_container: name

     # Optional additional container layers on top of the
     # specified base container
     layers:
       - name_A
       - name_B

     # Optional registration setup
     # Container runtime parameters
     runtime:
       # Run the container engine as a user other than the
       # default target user root. The user may be either
       # a user name or a numeric user-ID (UID) prefixed
       # with the ‘#’ character (e.g. #0 for UID 0). The call
       # of the container engine is performed by sudo.
       # The behavior of sudo can be controlled via the
       # file /etc/sudoers
       runas: root

       # Resume the container from previous execution.
       # If the container is still running, the app will be
       # executed inside of this container instance.
       #
       # Default: false
       resume: true|false

       # Attach to the container if still running, rather than
       # executing the app again. Only makes sense for interactive
       # sessions like a shell running as app in the container.
       #
       # Default: false
       attach: true|false

       # Caller arguments for the podman engine in the format:
       # - PODMAN_OPTION_NAME_AND_OPTIONAL_VALUE
       # For details on podman options please consult the
       # podman documentation.
       # Example:
       podman:
         - --storage-opt size=10G
         - --rm
         - -ti

After reading of the app configuration information the application
will be called using the configured engine. If no runtime
arguments exists, the following defaults will apply:

- The instance will be removed after the call
- The instance allows for interactive shell sessions

All caller arguments will be passed to the program call inside
of the instance except for arguments that starts with the '@'
sign. Caller arguments of this type are only used in the instance
ID file name but will not be passed to the program call inside of
the instance. This allows users to differentiate the same
program call between different instances when using
a resume based flake setup.

DEBUGGING
---------

oci-pilot provides more inner works details if the following
environment variable is set:

.. code:: bash

   export PILOT_DEBUG=1

FILES
-----

* /usr/share/flakes
* /etc/flakes

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
