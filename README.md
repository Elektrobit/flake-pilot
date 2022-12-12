# OCI Pilot

oci-pilot is a very simple software to manage and run applications
that are actually provided inside of OCI container(s). There are
two main components:

1. oci-pilot

   The launcher binary. Every application on your host that actually
   calls an application inside of a container is redirected to this
   launcher binary. oci-pilot currently only supports ```podman```
   as the runtime engine for containers.

2. oci-ctl

   The management utility to list, register, remove, more...
   container applications on your host.

## Installation

```oci-pilot``` and ```oci-ctl``` are two implementations written in ```rust```
They can be compiled one after the other as follows:

```bash
pushd oci-pilot
cargo build --release
popd
pushd oci-ctl
cargo build --release
popd
sudo install -m 755 oci-pilot/target/release/oci-pilot /usr/bin
sudo install -m 755 oci-ctl/target/release/oci-pilot /usr/bin
```

## Quick Start

As a start let's register an application named ```aws``` which is
connected to the ```aws-cli``` container provided by Amazon on
```docker.io/amazon```.

1. Pull the container

   ```bash
   podman pull docker.io/amazon/aws-cli
   ```

2. Register the ```aws``` application

   ```bash
   oci-ctl register --container amazon/aws-cli --app /usr/bin/aws --target /
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

## Application Setup

After the registration of an application they can be listed via

```bash
oci-ctl list
```

Each application provides a configuration below ```/usr/share/flakes/```
The term ```flake``` is a short name that we came up with to simplify
the conversations when talking about *container application registrations*
:-) For our above registered ```aws``` flake the config file structure
looks like the following:

```
/usr/share/flakes/
├── aws.d
└── aws.yaml
```

Please consult the manual pages for detailed information about the
contents of the flake setup.

oci-pilot is a project in its early stages and the result of
a fun conversation over beer on a conference. Feedback
is very much welcome.

Remember to have fun :)
