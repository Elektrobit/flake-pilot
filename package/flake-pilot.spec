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
Version:        2.1.1
Release:        0
Summary:        flake-pilot - launcher for flake applications
License:        MIT
%if "%{_vendor}" == "debbuild"
Packager:       Marcus Schaefer <marcus.schaefer@elektrobit.com>
%endif
Group:          Application/Misc
Url:            https://github.com/schaefi/pilot
Source0:        %{name}.tar.gz
Source1:        cargo_config
%if 0%{?debian} || 0%{?ubuntu}
Requires:       golang-github-containers-common
%endif
Requires:       podman
Requires:       sudo
Requires:       rsync
Requires:       tar
BuildRequires:  pandoc
%if 0%{?fedora} || 0%{?suse_version}
BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  upx
%endif
%if 0%{?debian} || 0%{?ubuntu}
BuildRequires:  rust-all
BuildRequires:  upx-ucl
%endif
BuildRoot:      %{_tmppath}/%{name}-%{version}-build

%description
Run flake applications using a symlink structure pointing
to a launcher binary which actually launches the application through
a runtime engine like podman. Along with the launcher there is
also a control tool to register an application as a flake application

%package -n oci-deb
Summary:        oci-deb - build flake-pilot compliant debian package from OCI container image
Group:          Application/Misc
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

%prep
%setup -q -n flake-pilot

%build
mkdir -p podman-pilot/.cargo
mkdir -p flake-ctl/.cargo
cp %{SOURCE1} podman-pilot/.cargo/config
cp %{SOURCE1} flake-ctl/.cargo/config
make build

%install
make DESTDIR=%{buildroot}/ install
chmod 777 %{buildroot}/usr/share/flakes


%files
%defattr(-,root,root)
%dir /usr/share/flakes
%dir /etc/flakes
/etc/flakes/container-flake.yaml
/usr/bin/podman-pilot
/usr/bin/flake-ctl
/usr/sbin/oci-registry
%doc /usr/share/man/man8/*

%files -n oci-deb
/usr/share/podman-pilot
/usr/bin/oci-deb

%changelog
