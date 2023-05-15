#
# spec file for package flake-pilot
#
# Copyright (c) 2022 Elektrobit Automotive GmbH
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in
# all copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#
Name:           flake-pilot
Version:        2.1.13
Release:        0
Summary:        Launcher for flake applications
License:        MIT
%if "%{_vendor}" == "debbuild"
Packager:       Marcus Schaefer <marcus.schaefer@elektrobit.com>
%endif
Group:          System/Management
Url:            https://github.com/schaefi/pilot
Source0:        %{name}.tar.gz
Source1:        cargo_config
%if 0%{?debian} || 0%{?ubuntu}
Requires:       golang-github-containers-common
%endif
Requires:       sudo
Requires:       rsync
Requires:       tar
BuildRequires:  pandoc
%if 0%{?fedora} || 0%{?suse_version}
BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  upx
BuildRequires:  openssl-devel
%endif
%if 0%{?debian} || 0%{?ubuntu}
BuildRequires:  rust-all
BuildRequires:  upx-ucl
BuildRequires:  libssl-dev
BuildRequires:  openssl
BuildRequires:  pkg-config
%endif
BuildRoot:      %{_tmppath}/%{name}-%{version}-build

%description
Run flake applications using a symlink structure pointing
to a launcher binary which actually launches the application through
a runtime engine like podman. Along with the launcher there is
also a control tool to register an application as a flake application

%package -n oci-deb
Summary:        Build flake-pilot compliant debian package from OCI container image
Group:          System/Management
%if 0%{?debian} || 0%{?ubuntu}
Requires:       libxml2-utils
%else
Requires:       libxml2-tools
%endif
Requires:       rsync
Requires:       dpkg
Requires:       dpkg-dev
Requires:       debbuild

%description -n oci-deb
Provides oci-deb utility which uses debbuild and dpkg to create
a debian package from a given OCI image file. The created debian
package hooks into the flake-pilot registration mechanism to run
containerized applications.

%package -n flake-pilot-podman
Summary:        Podman pilot
Group:          System/Management
Requires:       rsync
Requires:       podman

%description -n flake-pilot-podman
Launcher for OCI containers based applications through podman

%package -n flake-pilot-firecracker
Summary:        FireCracker pilot
Group:          System/Management
%if 0%{?debian} || 0%{?ubuntu}
BuildRequires:  clang
%else
BuildRequires:  clang-devel
%endif
Requires:       rsync
Requires:       firecracker
Requires:       xz
Requires:       e2fsprogs
%if 0%{?suse_version}
Requires:       qemu-tools
%else
Requires:       qemu-img
%endif

%description -n flake-pilot-firecracker
Launcher and service tools for KVM VM based applications
through firecracker

%package -n flake-pilot-firecracker-dracut-netstart
Summary:        Dracut Module Network Startup
Group:          System/Management
%if 0%{?suse_version}
Requires:       systemd-network
%else
Requires:       systemd
%endif

%description -n flake-pilot-firecracker-dracut-netstart
Start systemd network and resolver inside of the initrd such
that the network setup persists after switch_root if there
is no systemd process called but sci as simple command
execution interface

%package -n flake-pilot-firecracker-guestvm-tools
Summary:        FireCracker guest VM tools
Group:          System/Management

%description -n flake-pilot-firecracker-guestvm-tools
Guest VM tools to help with firecracker workloads

%prep
%setup -q -n flake-pilot

%build
mkdir -p podman-pilot/.cargo
mkdir -p firecracker-pilot/.cargo
mkdir -p flake-ctl/.cargo
mkdir -p firecracker-pilot/firecracker-service/service/.cargo
mkdir -p firecracker-pilot/guestvm-tools/sci/.cargo
cp %{SOURCE1} podman-pilot/.cargo/config
cp %{SOURCE1} firecracker-pilot/.cargo/config
cp %{SOURCE1} flake-ctl/.cargo/config
cp %{SOURCE1} firecracker-pilot/firecracker-service/service/.cargo/config
cp %{SOURCE1} firecracker-pilot/guestvm-tools/sci/.cargo/config
make build

%install
make DESTDIR=%{buildroot}/ install
chmod 777 %{buildroot}/usr/share/flakes

mkdir -p %{buildroot}/overlayroot

mkdir -p %{buildroot}/var/lib/firecracker/images
chmod 777 %{buildroot}/var/lib/firecracker/images

mkdir -p %{buildroot}/var/lib/firecracker/storage
chmod 777 %{buildroot}/var/lib/firecracker/storage

mkdir -p %{buildroot}/etc/dracut.conf.d
mkdir -p %{buildroot}/usr/lib/dracut/modules.d/80netstart
cp -a firecracker-pilot/dracut/usr/lib/dracut/modules.d/80netstart/* \
    %{buildroot}/usr/lib/dracut/modules.d/80netstart
install -m 644 firecracker-pilot/dracut/etc/dracut.conf.d/extramodules.conf \
    %{buildroot}/etc/dracut.conf.d/extramodules.conf

%files
%defattr(-,root,root)
%dir /usr/share/flakes
%dir /etc/flakes
/usr/bin/flake-ctl
%doc /usr/share/man/man8/flake-ctl.8.gz
%doc /usr/share/man/man8/flake-ctl-list.8.gz

%files -n flake-pilot-podman
%config /etc/flakes/container-flake.yaml
/usr/bin/podman-pilot
/usr/sbin/oci-registry
%doc /usr/share/man/man8/flake-ctl-podman-build-deb.8.gz
%doc /usr/share/man/man8/flake-ctl-podman-load.8.gz
%doc /usr/share/man/man8/flake-ctl-podman-pull.8.gz
%doc /usr/share/man/man8/flake-ctl-podman-register.8.gz
%doc /usr/share/man/man8/flake-ctl-podman-remove.8.gz
%doc /usr/share/man/man8/podman-pilot.8.gz

%files -n flake-pilot-firecracker
%dir /var/lib/firecracker
%dir /var/lib/firecracker/images
%dir /var/lib/firecracker/storage
%config /etc/flakes/firecracker-flake.yaml
%config /etc/flakes/firecracker.json
%doc /usr/share/man/man8/flake-ctl-firecracker-pull.8.gz
%doc /usr/share/man/man8/flake-ctl-firecracker-remove.8.gz
%doc /usr/share/man/man8/flake-ctl-firecracker-register.8.gz
/usr/bin/firecracker-service
/usr/bin/firecracker-pilot
%doc /usr/share/man/man8/firecracker-service.8.gz
%doc /usr/share/man/man8/firecracker-pilot.8.gz

%files -n flake-pilot-firecracker-dracut-netstart
%dir /usr/lib/dracut
%dir /usr/lib/dracut/modules.d
%dir /usr/lib/dracut/modules.d/80netstart
%dir /etc/dracut.conf.d
/usr/lib/dracut/modules.d/80netstart
/etc/dracut.conf.d/extramodules.conf

%files -n flake-pilot-firecracker-guestvm-tools
%dir /overlayroot
/usr/sbin/sci

%files -n oci-deb
/usr/share/podman-pilot
/usr/bin/oci-deb
