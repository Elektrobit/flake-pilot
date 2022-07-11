Name:           pilot
Version:        0.0.1
Release:        0
Summary:        pilot - launcher for container applications
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
to pilot which actually launches the application through podman

%prep
%setup -q -n pilot

%build
cd launcher
mkdir .cargo
cp %{SOURCE2} .cargo/config
cargo build --release

%install
mkdir -p %{buildroot}/usr/sbin
install -m 755 launcher/target/release/pilot %{buildroot}/usr/sbin

%files
%defattr(-,root,root)
/usr/sbin/pilot

%changelog
