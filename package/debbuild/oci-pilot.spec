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
BuildRequires:  rust-all
BuildRoot:      %{_tmppath}/%{name}-%{version}-build

%description
Run container applications using a symlink structure pointing
to oci-pilot which actually launches the application through podman.
Along with the launcher there are also registration tools to
manage the symlink structure and podman registry

%prep
%setup -q -n oci-pilot

%build
mkdir -p oci-pilot/.cargo
mkdir -p oci-register/.cargo
cp %{SOURCE2} oci-pilot/.cargo/config
cp %{SOURCE2} oci-register/.cargo/config
make build

%install
make DESTDIR=%{buildroot}/ install

%files
%defattr(-,root,root)
/usr/bin/oci-pilot
/usr/bin/oci-register

%changelog