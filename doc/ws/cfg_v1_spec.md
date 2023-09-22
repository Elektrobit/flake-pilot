# Flakes Configuration (v1)

This is the specs of the configuration for Flakes up to 2.2.19 version.

Status: **deprecated**

## Podman

Configuration sample for Flakes, running OCI containers.

```yaml
container:
  name: name
  target_app_path: path/to/program/in/container
  host_app_path: path/to/program/on/host

  # Optional base container to use with a delta 'container: name'
  # If specified the given 'container: name' is expected to be
  # an overlay for the specified base_container. podman-pilot
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

    podman:
      - --storage-opt size=10G
      - --rm
      - -ti

include:
  tar:
    - tar-archive-file-name-to-include
```

## Firecracker VM

Configuration sample for Flakes running a VM.

```yaml
vm:
  name: name
  target_app_path: path/to/program/in/VM
  host_app_path: path/to/program/on/host

  runtime:
    # Run the VM engine as a user other than the
    # default target user root. The user may be either
    # a user name or a numeric user-ID (UID) prefixed
    # with the ‘#’ character (e.g. #0 for UID 0). The call
    # of the VM engine is performed by sudo.
    # The behavior of sudo can be controlled via the
    # file /etc/sudoers
    runas: root

    # Resume the VM from previous execution.
    # If the VM is still running, the app will be
    # executed inside of this VM instance.
    #
    # Default: false
    resume: true|false

    firecracker:
      # Currently fixed settings through app registration
      boot_args:
        - "init=/usr/sbin/sci"
        - "console=ttyS0"
        - "root=/dev/vda"
        - "acpi=off"
        - "rd.neednet=1"
        - "ip=dhcp"
        - "quiet"
      mem_size_mib: 4096
      vcpu_count: 2
      cache_type: Writeback

      # Size of the VM overlay
      # If specified a new ext2 overlay filesystem image of the
      # specified size will be created and attached to the VM
      overlay_size: 20GiB

      # Path to rootfs image done by app registration
      rootfs_image_path: /var/lib/firecracker/images/NAME/rootfs

      # Path to kernel image done by app registration
      kernel_image_path: /var/lib/firecracker/images/NAME/kernel

      # Optional path to initrd image done by app registration
      initrd_path: /var/lib/firecracker/images/NAME/initrd
```
