OCI-PILOT(8)
============

NAME
----

**oci-pilot** - Launcher for container applications

DESCRIPTION
-----------

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
can be set:

.. code:: yaml

   # Mandatory registration setup
   # Name of the container in the local registry
   container: name

   # Path of the program to call inside of the container (target)
   target_app_path: path/to/program/in/container

   # Path of the program to register on the host
   host_app_path: path/to/program/on/host

   # Optional registration setup
   # Container runtime parameters
   runtime:
     # Try to resume container from previous execution.
     # If the container is still running, the call will attach to it
     # If the container is not running, the call will restart the
     # container and attach to it.
     #
     # NOTE: If processing the call inside of the container finishes
     # faster than attaching to it, the call will run a new container
     # which is attached immediately. If this is unwanted set:
     # respawn: false
     #
     # Default: false
     resume: true|false

     # Run a new container if attaching to resumed container failed
     # This setting is only effective if 'resume: true' is set
     #
     # Default: true
     respawn: true|false

     # Caller arguments for the podman engine in the format:
     # - PODMAN_OPTION_NAME_AND_OPTIONAL_VALUE
     # For details on podman options please consult the
     # podman documentation.
     # Example:
     podman:
       - --storage-opt size=10G
       - --rm
       - -ti

After reading of the app configuration information the container
will be called using the configured engine. If no runtime
arguments for the container call are provided, the following
defaults will apply:

- The container will be removed after the call
- The container allows for interactive shell sessions

All caller arguments will be passed to the program call inside
of the container except for arguments that starts with the '@'
sign. Caller arguments of this type are only used in the container
ID file name but will not be passed to the program call inside of
the container. This allows users to differentiate the same
program call between different container instances when using
a resume based flake setup.

FILES
-----

* /usr/share/flakes

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
