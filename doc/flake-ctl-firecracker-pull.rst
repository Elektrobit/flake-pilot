FLAKE-CTL-FIRECRACKER-PULL(8)
=============================

NAME
----

**flake-ctl firecracker pull** - Fetch firecracker image

SYNOPSIS
--------

.. code:: bash

   USAGE:
       flake-ctl firecracker pull [OPTIONS] --name <NAME> --kis-image <KIS_IMAGE>

   OPTIONS:
       --force
       --kis-image <KIS_IMAGE>
       --name <NAME>

DESCRIPTION
-----------

Pull the components of a firecracker image from the given location
into `/var/lib/firecracker/images/NAME` on the local machine.
After completion the available firecracker images can be listed via:

.. code:: bash

   $ tree /var/lib/firecracker/images

and shows a file structure like in the following example

.. code:: bash

   /var/lib/firecracker/images
   └── myImage
        ├── initrd
        ├── kernel
        └── rootfs

OPTIONS
-------

--force

  Force pulling the image even if it already exists This will wipe
  existing data for the provided identifier

--kis-image <KIS_IMAGE>

  Firecracker image built by KIWI as kis image type to pull
  into local image store. This means the file behind KIS_IMAGE
  is expected to be a tarball containing the KIS
  components; rootfs-image, kernel and optional initrd

--name <NAME>

  Image name used as local identifier

EXAMPLE
-------

.. code:: bash

   $ flake-ctl firecracker pull --name myImage --kis-image \
       https://download.opensuse.org/repositories/home:/marcus.schaefer:/delta_containers/images/firecracker-basesystem.x86_64.tar.xz

AUTHOR
------

Marcus Schäfer

COPYRIGHT
---------

(c) 2022, Elektrobit Automotive GmbH
