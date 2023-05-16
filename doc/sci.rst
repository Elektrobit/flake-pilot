SCI(8)
======

NAME
----

**sci** - Execute provided command in virtualized environment

SYNOPSIS
--------

.. code:: bash

   USAGE:
       sci

   OPTIONS:
       *none*


DESCRIPTION
-----------

**NOTE**
    sci is not meant to be called as a user tool. At the end of execution
    sci will send a sysreq signal. sci is meant to be exclusively executed 
    inside a firecracker instance!


Simple Command Init (sci) is a tool which executes the provided
command in the run=... cmdline variable after preparation of an
execution environment for the purpose to run a command inside
of a firecracker instance.

Inside the fircracker.json configuration file kernel boot parameters
can be provided. Here various environment variables can be set.
Available variables are:


    + run= command
    + overlay_root= /dev/block_device


If provided via the overlay_root=/dev/block_device kernel boot
parameter in the firecracker.json file,
sci also prepares the root filesystem as an overlay
using the given block device for writing.

For the overlay_root parameter to work the firecracker.json file
needs to have a proper section with
a record of the overlayfs on the root system.

Every environment variable configurable and all options 
regarding filesystems are stored in the firecracker.json
file for the individual instance. Having this in mind, the desired values should
ideally be set in the belonging file.

.. _repository: https://github.com/Elektrobit/flake-pilot/tree/master/firecracer-pilot/template

For a working example refer to the firecracker.json template at the offical
repository_



ENVIROMENT VARIABLES
--------------------

+----------------------+-------------------+----------------------------------+
| Variable             | Value             | Description                      |       
+======================+===================+==================================+
|                      |                   |                                  |
|                      |                   |                                  |
| run                  | command           | sci will replace init and        |
|                      |                   | execute the provided command     |
|                      |                   | at startup                       |
|                      |                   |                                  |
+----------------------+-------------------+----------------------------------+
|                      |                   |                                  |
|overlay_root          | /dev/block_device | if the rootfs is read only       |
|                      |                   | an overlay is required to        |
|                      |                   | write to the filesystem.         |
|                      |                   | Each application will maintain   |
|                      |                   | their own specific overlay.      |
|                      |                   | Changes to rootfs will be        |
|                      |                   | stored in the overlay and applied|
|                      |                   | to the individual rootfs.        |
|                      |                   | Changes to the original rootfs   |
|                      |                   | will not be made.                |
|                      |                   |                                  |
+----------------------+-------------------+----------------------------------+

FILES
-----

* /usr/sbin/sci

NOTION
------

sci will execute these steps in order:

    + evaluation of environment variable 'run'
    + mounting of overlay if requested
    + switching root into overlay if configured
    + execution of provided command
    + reboot of firecracker instance



AUTHOR
------

André Barthel

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
