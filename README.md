# Flake Pilot

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Quick Start OCI containers](#oci)
    1. [Use Cases](#usecases)
3. [Quick Start FireCracker VMs](#fire)

## Introduction <a name="introduction"/>

flake-pilot is a software to register, provision and launch applications
that are actually provided inside of a runtime environment like an
OCI container or a FireCracker VM. There are two main components:

1. The launchers

   The launcher binary. Each application that was registered as a
   flake is redirected to a launcher binary. As of today
   support for the ```podman``` and ```firecracker``` engines are
   implemented leading to the respective ```podman-pilot``` and
   ```firecracker-pilot``` launcher binaries.

2. The flake registration tool

   ```flake-ctl``` is the management utility to list, register,
   remove, and-more... flake applications on your host.

## Installation <a name="installation"/>

flake-pilot components are written in rust and available as packages
here: https://build.opensuse.org/package/show/home:marcus.schaefer:delta_containers/flake-pilot

Manual compilation and installation can be done as follows:

```bash
make build && make install
```

## Quick Start OCI containers <a name="oci"/>

As a start let's register an application named ```aws``` which is
connected to the ```aws-cli``` container provided by Amazon on
```docker.io/amazon```.

1. Pull the container

   ```bash
   flake-ctl podman pull --uri docker.io/amazon/aws-cli
   ```

2. Register the ```aws``` application

   ```bash
   flake-ctl podman register --container amazon/aws-cli --app /usr/bin/aws --target /
   ```

   This creates ```/usr/bin/aws``` on your host which actually
   launches the ```amazon/aws-cli``` container. The default entry
   point of the container was configured by Amazon to launch their
   cloud API application. Thus the target program to call inside
   of the container doesn't need to be explicitly configured in
   the registration and is therefore just set to ```/```

3. Launch the application

   To run ```aws``` just call for example:

   ```bash
   aws ec2 help
   ```

### Use Cases <a name="usecases"/>

Apart from this very simple example you can do a lot more. The main
idea for flake-pilot was not only to launch container based apps but
also allow to run a provision step prior calling the application.
This concept then allows for use cases like

* delta containers used together with a base container such that
  only small delta containers gets pulled to the registry used with
  a base that exists only once.

* include arbitrary data without harming the host integrity e.g custom
  binaries, proprietary software not following package guidelines and
  standards, e.g automotive industry processes which we will not be
  able to change in this live ;)

* layering of several containers, e.g deltas on top of a base. Building
  up a solution stack e.g base + python + python-app.

* provisioning app dependencies from the host instead of providing them
  in the container, e.g a delta container providing the app using a base
  container but take the certificates or other sensitive information
  from the host; three way dependency model.

Actually all of the above use cases are immaterial if a proper packaging,
release and maintenance of the application is possible. However, I have
learned the world is not an ideal place and there might be a spot for
this project to be useful, supporting users with "special" needs and
adding an adaptive feature to the OS.

For demo purposes and to showcase the mentioned use cases, the following
Open Build Service project was created: https://build.opensuse.org/package/show/home:marcus.schaefer:delta_containers
Feel free to browse through the project and have some fun testing. There
is a short description in each package how to use them.

**_NOTE:_** Key to success is also the ability to build the individual containers
and VM images. This is achieved by using the [KIWI](https://github.com/OSInside/kiwi)
appliance builder which is supported by the Open Build Service backend
and allows to build all the images in a nice way.

## Quick Start FireCracker VMs <a name="fire"/>

Using containers to isolate applications from the host system is a common approach.
The limitation comes on the level of the kernel. Each container shares the kernel
with the host and if applications requires to run privileged, requires direct access
to device nodes or kernel interfaces like the device mapper, a deeper level of
isolation might be needed. At this point full virtual system instances running its
own kernel, optional initrd and processes inside provides a solution. The price
to pay is on the performance side but projects like KVM and FireCracker offers
a nice concept to run virtual machines accelerated through KVM as competitive
alternative to containers. Thus flake-pilot also implements support for the
firecracker engine.

Start an application as virtual machine (VM) instance as follows:

1. Pull a firecracker compatible VM

   ```bash 
   flake-ctl firecracker pull --name leap --kis-image https://download.opensuse.org/repositories/home:/marcus.schaefer:/delta_containers/images/firecracker-basesystem.x86_64.tar.xz
   ```

2. Register the ```mybash``` application
  
   ```bash
   flake-ctl firecracker register --vm leap --app /usr/bin/mybash --target /bin/bash --overlay-size 20g
   ```

This registers an app named ```mybash``` to the system. Once called a firecracker VM based on
the pulled ```leap``` image is started and the ```/bin/bash``` program is called inside of
the VM instance. In addition some write space of 20GB is added to the instance

3. Launch the application

   To run ```mybash``` just call for example:

   ```bash
   mybash
   ```

   Drops you into a bash shell inside of the VM

## Application Setup

After the registration of an application they can be listed via

```bash
flake-ctl list
```

Each application provides a configuration below ```/usr/share/flakes/```.
The term ```flake``` is a short name that we came up with to provide
a generic name for an application running inside of an isolated environment.
For our above registered ```aws``` flake the config file structure
looks like the following:

```
/usr/share/flakes/
├── aws.d
└── aws.yaml
```

Please consult the manual pages for detailed information 
about the contents of the flake setup.

https://github.com/Elektrobit/flake-pilot/tree/master/doc

Flake pilot is a project in its early stages and the result of
a fun conversation over beer on a conference. Feedback
is very much welcome.

Remember to have fun :)
