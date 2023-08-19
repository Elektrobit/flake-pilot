*** Settings ***
Library    Process
*** Test Cases ***
Podman is installed
    ${result} =  Run Process  podman  --version
    Log  ${result.stdout}
    Log  ${result.stderr}
    Should Be Equal As Integers    ${result.rc}    0
