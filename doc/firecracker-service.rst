FIRECRACKER-SERVICE(8)
======================

NAME
----

**firecracker-service** - firecracker instance handler

SYNOPSIS
--------

.. code:: bash

    firecracker-service

DESCRIPTION
-----------

firecracker-service exists to support users controlling flake applications started through the
firecracker engine. In contrast to e.g container tools like podman, firecracker does not yet
provide an infrastructure to manage (list, start, stop, etc...) firecracker instances. The service
is running as a daemon and currently provides the possibility to register and store information about current
running firecracker micro vm's. Gives also possibility to query the service about current running vm's. 
The communication endpoint to the service is the unix socket **/run/firecracker-service.socket** where commands 
can be send in json format.

Command send to the service

.. code:: json

    {
        "name": "command_name",
        "vm": "optional information about vm if needed"
    }

Commands allowed are:
* ps - return a list of currently running vm's
* register - register new running instance, vm field is needed
* unregister - notify that vm instance has closed recently, vm field is needed

Virtual machine object is build as:

.. code:: json

    {
        "id":"some_id",
        "cmd": ["/bin/bash","-c","example_command"]
    }

Each command call returns a Response object that returns if the operation succeeded in 
ok field. If operation failed, the additional information is stored in optional field **error_msg**.
The result can carry optional data such as an array of Vitual machine objects. 
Examplary failed operation result can look like:

.. code:: json

    {
        "ok": false,
        "error_msg": "Wrong formatted command call"
    }

Examplary result of **ps** command, which returns a list of currently running Vm's can look
like this:

.. code:: json

    {
        "ok":true,
        "vm_list": [
            { "id": "vm1", "cmd": ["some","command", "with", "params"] },
            { "id": "vm2", "cmd": ["other","command", "with", "params"] },
        ]
    }

FILES
-----

* /usr/sbin/firecracker-service
* /run/firecracker-service.socket

EXAMPLE
-------

.. code:: bash

   $ firecracker-service

AUTHOR
------

Marcin Katulski

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
