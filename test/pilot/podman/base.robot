*** Settings ***
Library    Process
Library    OperatingSystem
Test Teardown  Teardown
*** Test Cases ***
Hello World
    Register Container    registry.opensuse.org/home/marcus.schaefer/delta_containers/containers_ubuntu/ubuntu:latest    hello_world
    ${result} =  Run Process    sudo  /usr/bin/hello_world  echo  Hello World
    Should Be Equal As Strings    ${result.rc}    0
    Should Be Equal As Strings    ${result.stdout}    Hello World

    [Teardown]  Run Process    sudo  rm  -r  /usr/bin/hello_world

*** Keywords ***
Register Container
    [Arguments]    ${container}    ${name}
    ${result} =    Run Process    sudo
    ...    flake-ctl    podman    register
    ...    --container    ${container}
    ...    --app    /usr/bin/${name}
    ...    --target    /

    Log    ${result.stdout}
    Log    ${result.stderr}
    Should Be Equal As Strings    ${result.rc}    0
    File Should Exist    /usr/share/flakes/${name}.yaml
    RETURN    ${result}


Teardown
    Run Process    sudo  rm  -r  /usr/share/flakes
    Run Process    podman  prune  -a  -f
