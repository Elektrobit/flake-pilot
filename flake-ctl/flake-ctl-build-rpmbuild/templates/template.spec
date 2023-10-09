# Only edit if you know what you are doing
Name:           %{_flake_name}
Version:        %{_flake_version}
Release:        1%{?dist}
Summary:        Lorem Ipsum 
BuildArch:      noarch
License:        %{_flake_license}
Maintainer:     "%{_flake_maintainer_name}" <%{_flake_maintainer_email}>

%{_flake_requires}

## Only edit the lines below if you *really* know what you are doing ##
Source0:        %{name}.tar.gz

%description
A demo RPM build

%prep
%setup -q

%install
rm -rf $RPM_BUILD_ROOT
mkdir -p $RPM_BUILD_ROOT/usr/share/flakes
mkdir -p $RPM_BUILD_ROOT/tmp
cp ./%{_flake_dir}/%{name}.yaml $RPM_BUILD_ROOT/%{_flake_dir}
cp ./%{_flake_dir}/%{name}.d $RPM_BUILD_ROOT/%{_flake_dir} -r
cp %{name} $RPM_BUILD_ROOT/tmp

%post
podman load < /tmp/%{name}
ln -s %{_bindir}/%{_flake_pilot}-pilot %{_bindir}/%{name}

%postun
rm %{_bindir}/%{name}

%clean
rm -rf $RPM_BUILD_ROOT

%files
/tmp/%{name}
/%{_flake_dir}/%{name}.yaml
/%{_flake_dir}/%{name}.d
