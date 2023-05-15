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

Simple Command Init (sci) is a tool which executes the provided
command in the run=... cmdline variable after preparation of an
execution environment for the purpose to run a command inside
of a firecracker instance.

Inside the fircracker JSON configuration file kernel boot parameters
can be provided. Here various environment variables can be set.
Available variables are:


    + run= command
    + overlay_root= /dev/block_device


if provided via the overlay_root=/dev/block_device kernel boot
parameter, sci also prepares the root filesystem as an overlay
using the given block device for writing.


ENVIROMENT VARIABLES
--------------------

+----------------------+-------------------+----------------------------------+
| Variable             | Value             | Description                      |       
+======================+===================+==================================+
|                      |                   |                                  |
|                      |                   |                                  |
| run                  | command           | sci will replace initd and       |
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

EXAMPLE
-------

**fircracker.json**::


{
 "boot-source":{
   "boot_args": overlay_root=/dev/vdb interactive=true init=/sbin/sci run=\\"bash -c 'ls -la ./*'\\"
  },
 }


AUTHOR
------

André Barthel

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
