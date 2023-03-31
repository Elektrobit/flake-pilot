# Flake Pilot

flake-pilot is a simple software to register and launch applications
that are actually provided inside of a runtime environment like an
OCI container. There are two main components:

1. The launcher(s)

   The launcher binary. Each application that was registered as a
   flake is redirected to a launcher binary. As of today only
   support for the ```podman``` engine is implemented leading to
   the respective ```podman-pilot``` launcher binary.

2. The flake registration tool

   ```flake-ctl``` is the management utility to list, register,
   remove, more... flake applications on your host.

## Installation

launcher(s) and control tool are implementations written in ```rust```.
They can be compiled and installed one after the other as follows:

```bash
pushd podman-pilot
cargo build --release
popd
pushd flake-ctl
cargo build --release
popd
sudo mkdir -p /etc/flakes
sudo install -m 644 flake-ctl/template/container-flake.yaml /etc/flakes
sudo install -m 755 podman-pilot/target/release/podman-pilot /usr/bin
sudo install -m 755 flake-ctl/target/release/flake-ctl /usr/bin
```

## Quick Start

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

Flake pilot is a project in its early stages and the result of
a fun conversation over beer on a conference. Feedback
is very much welcome.

Remember to have fun :)
