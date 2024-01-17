# Only edit if you know what you are doing
Name:           %{_flake_package_name}
Version:        %{_flake_version}
Release:        1%{?dist}
Summary:        Lorem Ipsum 
BuildArch:      noarch
License:        %{_flake_license}
%if "%{_vendor}" == "debbuild"
Maintainer:     "%{_flake_maintainer_name}" <%{_flake_maintainer_email}>
%endif

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
pwd
cp ./%{_flake_dir}/%{_flake_name}.yaml $RPM_BUILD_ROOT/%{_flake_dir}
cp ./%{_flake_dir}/%{_flake_name}.d $RPM_BUILD_ROOT/%{_flake_dir} -r
cp %{_flake_name} $RPM_BUILD_ROOT/tmp

%post
podman load < /tmp/%{name}
%{_flake_links_create}

%postun
%{_flake_links_remove}
podman rmi %{_flake_image_tag}

%clean
rm -rf $RPM_BUILD_ROOT

%files
/tmp/%{_flake_name}
/%{_flake_dir}/%{_flake_name}.yaml
/%{_flake_dir}/%{_flake_name}.d
