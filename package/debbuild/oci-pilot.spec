#
# spec file for package oci-pilot
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
Name:           oci-pilot
Version:        0.0.1
Release:        0
Summary:        oci-pilot - launcher for container applications
License:        MIT
%if "%{_vendor}" == "debbuild"
Packager:       Marcus Schaefer <marcus.schaefer@elektrobit.com>
%endif
Group:          Application/Misc
Url:            https://github.com/schaefi/pilot
Source0:        %{name}.tar.gz
Source1:        %{name}-vendor.tar.gz
Source2:        cargo_config
Requires:       golang-github-containers-common
Requires:       podman
BuildRequires:  rust-all
BuildRequires:  pandoc
BuildRequires:  upx-ucl
BuildRoot:      %{_tmppath}/%{name}-%{version}-build

%description
Run container applications using a symlink structure pointing
to oci-pilot which actually launches the application through podman.
Along with the launcher there are also registration tools to
manage the symlink structure and podman registry

%package -n oci-deb
Summary:        oci-deb - build oci-pilot compliant debian package from OCI tar
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
a debian package from a given OCI tar file. The created debian
package hooks into the oci-pilot registration mechanism to run
containerized applications.

%prep
%setup -q -n oci-pilot

%build
mkdir -p oci-pilot/.cargo
mkdir -p oci-ctl/.cargo
cp %{SOURCE2} oci-pilot/.cargo/config
cp %{SOURCE2} oci-ctl/.cargo/config
make build

%install
make DESTDIR=%{buildroot}/ install

%files
%defattr(-,root,root)
/usr/bin/oci-pilot
/usr/bin/oci-ctl
%doc /usr/share/man/man8/*

%files -n oci-deb
/usr/share/oci-pilot
/usr/bin/oci-deb

%changelog
