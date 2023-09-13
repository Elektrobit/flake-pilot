# Automated Packaging of Flakes

| Status        |  Proposed                                            |
:-------------- |:---------------------------------------------------- |
| **RFC #**     | (update when you have a PR #)                        |
| **Author(s)** | Michael Meyer (ichmed95@gmail.com)                   |
| **Sponsor**   |                                                      |
| **Updated**   | 2023-09-13                                           |

## Objective

Create a flake-ctl tool for automated packaging of flakes so users can create .deb or .rpm files from an existing flake with a single command.


## Motivation

Packaging is a common and straightforward use case for Flakes and all Flakes should be packaged in essentially the same fashion. But creating the structure needed to create a package can be tedious and error prone.

## User Benefit

- By providing a single streamlined packaging command users do not need to deal with the intricacies of packaging.


## Design Proposal

There will be a flake-ctl tool addon for each kind of package, e.g. `flake-ctl-pkg-deb` or `flake-ctl-pkg-rpm`.

These addons each have a dedicated folder where pilots can drop any needed information for that packaging.

Example:

```
# /etc/share/flake-ctl/pkg-rpm/podman
Require: podman
Require: podman-pilot
```

This way external pilots can provide their own information on how to package them.

Packaging tools shall be called like this
```
flake-ctl pkg-deb my_flake <args>
```
With the default arguments being:
- `--name` : set the name of the package (default: name of the flake, or name of the first delta argument)
- `--delta` : only include the listed entries from my_flake.d; allowed multiple times (default if none given: only include the base YAML)
- `--prepare` : only create the data needed to package the flake, without running the build step (used for OBS etc.)
- `--target` : Where to put the created package (or bundle in case of `--prepare`)

Specific packagers may provide further arguments.


### Alternatives Considered

- Provide templates for manual packaging
- oci-deb: There is existing code in `flake-ctl/flake-ctl-podman/deb` that uses oci-deb and may be repurposed. However it does not provide general functionality to package _any and all_ flakes, only oci based ones.
### Performance Implications

None since it is a separate executable

### Dependencies

Pilots need to bundle their packaging information somehow

### Engineering Impact

This tool will be maintained independently.

### Compatibility

This- design should be forward compatible since it does not actually interact with the contents of each flake apart from determining the engine of each flake. The API of this functionality will have to be ported with each change to the config format anyway.

### User Impact

None, the tool can be added into any existing installation

## Detailed Design

### Install and Uninstall
The created packages must make sure that they can be added/removed without disturbing any other flakes already present on the system.

Especially it is important that removing a flake does not modify or delete any exiting entries in the flake.d directory to guarantee smooth upgrading of flake versions. 

### Tool Information
When called with 
```
flake-ctl-pkg-<something> about
```
packagers should return 
```
Package flakes as <something> packages;PACKAGER
```

## Questions and Discussion Topics

- Does packaging merit its own overarching binary similar to flake-ctl? 
  
  (This would change `flake-ctl pkg-deb` to `flake-ctl pkg deb`). This could also be achieved by making `pkg` a builtin of `flake-ctl`.
- Should the packages rely on `flake-ctl`? 
  
  I see no benefit in using `flake-ctl` to register/remove the flakes. Instead the YAML can simply be provided and removed by the package. 
  
  Users can still use `flake-ctl` to manage flakes that were installed without the use of `flake-ctl`, but the dependency is removed from the package.
