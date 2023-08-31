*** Settings ***
Resource    ../../common/ctl.robot
*** Test Cases ***
Touch a file
    Register Podman Container    registry.opensuse.org/home/marcus.schaefer/delta_containers/containers_ubuntu/ubuntu:latest    he
    ...    working_dir=True
    ${result} =  Run Process    sudo  /usr/bin/he  touch  spaghett
    
    Log    ${result.stderr}
    Log    ${result.stdout}
    
    Should Be Equal As Strings    ${result.rc}    0

    File Should Exist    ./spaghett

    [Teardown]  Cleanup
*** Keywords ***
Cleanup
    Run Process    sudo  rm  -r  /usr/bin/he
    Remove File    ./spaghett
