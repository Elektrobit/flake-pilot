
1. [Installation](#installation)
2. [`flake-ctl`](#flake-ctl)
    1. [Runtimes](Runtimes)
    2. [`build`](#Build)
3. [Pilots](#Pilots)
4. [`flake-studio`](#flake-studio)
5. [Quickstart oci/podman](#oci)
6. [Quickstart firecracker](#fire)


`flake-pilot` is a software suite that enables you to provision, modify, package and launch containerized applications (such as oci containers) that can be run directly like any other command line utility.

This is accomplished creating symlinks to one of the lightweight `pilot` programs, which launch the corresponding runtime with all the arguments needed to mimic native behavior of the application as closely as possible.

# Installation

Manual compilation and installation can be done as follows:

```bash
make build && make install
```
# Flake-ctl
`flake-ctl` is the central command used to manage installed flakes. See `src/flake-ctl/README.md` for more detailed information

## Runtimes
Each supported runtime provides its own managing utility called `flake-ctl-<runtime>`. These must provide at least the commands 
- `register` which creates a new local flake
- `export` which exports all data needed to run a flake on another machine (e.g. the archived oci-container)

Beyond that the utilities may provide any number of additional sub commands

### Example
```bash
flake-ctl podman register ubuntu --container my_container --app /usr/bin/foo
```
This will create a local flake. Afterwards running `/usr/bin/foo` on will launch `my_container` using podman and run `/usr/bin/foo` inside of the container, forwarding any output back to the caller.

## Build
The `build` command is used to create packages that can be used to directly install
flakes on a system via a regular package manager. Right now the `build` command is only supported for oci-based flakes.

There are separated binaries for each supported package manager. The `flake-ctl-build` command will defer the actual build process to the native package manager (run `flake-ctl build which` to see which builder will be used).

### Example
```bash
# running on ubuntu
flake-ctl build which # dpkg-buildpackage;flake-ctl-build-dpkg
flake-ctl build --from-oci=my_container --target_app=/usr/bin/foo
```
When you run this command, a wizard will prompt you for further details needed to create the package. All of these parameters can also be supplied via
- command line (e.g. `--version`)
- environment variable (e.g. `PKG_FLAKE_VERSION`)
- inside a config file 
    - `./.flakes/package/options.yaml`
    - `~/.flakes/package/options.yaml`

After the process has finished it will place all produced files in you current working directory (or where specified with `--output`). The following would be produced by running the builder with `--name=foobar` and `--version=1.0.0`.
```
foobar_1.0.0_all.deb
foobar_1.0.0_amd64.buildinfo
foobar_1.0.0_amd64.changes
foobar_1.0.0.dsc
foobar_1.0.0.tar.gz
```
You can no install `foobar_1.0.0_all.deb` on another system using `dpkg`.

The package requires `podman-pilot` which in turn requires `podman`, if you do not have these packages installed on the system you will be prompted to run `apt install --fix-broken`. Alternatively you can install them manually.

<!--TODO: Include link to flake-pilot package 
the flake pilot package can be found [here](some.url.com)
-->
# Pilots
Currently there are two pilots:
- **podman-pilot** for oci containers using podman.
- **firecracker-pilot** for micro VMs

Pilots will not function if launched directly, instead they need to be called via
a symlink. When starting the pilot will use the symlinks name to retrieve a configuration (by default stored in `/usr/share/flakes`). This config contains all information needed to run the flake, including an image identifier and a list of command line parameters to the runtime environment.

Theses parameters can be tweaked to create a more seamless experience, for example by mounting directories from the host machine into the container to enable direct file access. 

The default configuration is optimized for one-shot applications that only communicate over stdin/out/err.
# Flake-studio

## Quick Start OCI containers <a name="oci"></a>

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


## Quick Start FireCracker VMs <a name="fire"></a>

Using containers to isolate applications from the host system is a common
approach. The limitation comes on the level of the kernel. Each container
shares the kernel with the host and if applications requires to run
privileged, requires direct access to device nodes or kernel interfaces
like the device mapper, a deeper level of isolation might be needed.
At this point full virtual system instances running its own kernel, optional
initrd and processes inside provides a solution. The price to pay is on
the performance side but projects like KVM and FireCracker offers a nice
concept to run virtual machines accelerated through KVM as competitive
alternative to containers. Thus flake-pilot also implements support for the
firecracker engine.

Start an application as virtual machine (VM) instance as follows:

1. Pull a firecracker compatible VM

   ```bash 
   flake-ctl firecracker pull --name leap \
       --kis-image https://download.opensuse.org/repositories/home:/marcus.schaefer:/delta_containers/images/firecracker-basesystem.x86_64.tar.xz
   ```

2. Register the ```mybash``` application
  
   ```bash
   flake-ctl firecracker register --vm leap \
       --app /usr/bin/mybash --target /bin/bash --overlay-size 20GiB
   ```

   This registers an app named ```mybash``` to the system. Once called a
   firecracker VM based on the pulled ```leap``` image is started and
   the ```/bin/bash``` program is called inside of the VM instance.
   In addition some write space of 20GB is added to the instance

3. Launch the application

   To run ```mybash``` just call for example:

   ```bash
   mybash
   ```

   Drops you into a bash shell inside of the VM

   **_NOTE:_** The data transfer from the virtual machine to the host
   is done through the serial console. As the process of calling the
   application includes the boot of the virtual machine, it might happen
   that kernel messages are intermixed with the output of the application.
   Our default setting prevents kernel messages from being printed to
   the console as much as possible but there are message that can hardly
   be prevented or requires a customized kernel build to be suppressed.
   As all messages are fetched from the serial console there is also
   no differentiation between **stdout** and **stderr** anymore.
