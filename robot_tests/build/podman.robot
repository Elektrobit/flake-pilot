*** Settings ***
Library         Process
Library         OperatingSystem
Resource        ../common/ctl.robot

Task Setup      Setup Env


*** Test Cases ***
Package Podman Image as .deb Package via dpkg
    Pull Podman Container    ubuntu
    Package an Image    dpkg    podman    ubuntu

Package Podman Image as .rpm Package via rpmbuild
    Pull Podman Container    ubuntu
    Package an Image    rpmbuild    podman    ubuntu


*** Keywords ***
Package a Flake
    [Arguments]    ${packager}    ${flake_name}
    ${result} =    Run Process    flake-ctl    build-${packager}    flake    ${flake_name}

Package an Image
    [Arguments]    ${packager}    ${pilot}    ${image_name}    ${app}=${TEMPDIR}/tmp_flake_app
    ${result} =    Run Process
    ...    flake-ctl-build-${packager}
    ...    --no-edit
    ...    image
    ...    ${pilot}
    ...    ${image_name}
    ...    ${app}
    ...    --target
    ...    ${TEMPDIR}/test_package

    Should Be Equal As Integers    ${result.rc}    0    ${result.stderr}
    Log    ${result.stderr}
    Log    ${result.stdout}

    File Should Exist    ${TEMPDIR}/test_package/TestPackage.deb

Setup Env
    Create Directory    ${TEMPDIR}/test_package
    Set Environment Variable    PKG_FLAKE_NAME    TestPackage
    Set Environment Variable    PKG_FLAKE_DESCRIPTION    A Test Package
    Set Environment Variable    PKG_FLAKE_VERSION    1.0.0
    Set Environment Variable    PKG_FLAKE_URL    example.com
    Set Environment Variable    PKG_FLAKE_MAINTAINER_NAME    Testian Tester
    Set Environment Variable    PKG_FLAKE_MAINTAINER_EMAIL    Testian@testing.com
    Set Environment Variable    PKG_FLAKE_LICENSE    MIT
