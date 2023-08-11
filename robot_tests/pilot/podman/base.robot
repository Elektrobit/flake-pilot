*** Settings ***
Library    Process
Library    OperatingSystem
Resource    ../../common/ctl.robot
Test Teardown  Teardown
*** Test Cases ***
Hello World
    Register Podman Container    registry.opensuse.org/home/marcus.schaefer/delta_containers/containers_ubuntu/ubuntu:latest    hello_world
    ${result} =  Run Process    sudo  /usr/bin/hello_world  echo  Hello World
    Should Be Equal As Strings    ${result.rc}    0
    Should Be Equal As Strings    ${result.stdout}    Hello World

    [Teardown]  Run Process    sudo  rm  -r  /usr/bin/hello_world

*** Keywords ***
Teardown
    Run Process    sudo  rm  -r  /usr/share/flakes
    Run Process    podman  prune  -a  -f
