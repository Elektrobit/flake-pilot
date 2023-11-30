*** Settings ***
Library    Process
*** Test Cases ***

Smoke
    ${result} =  Run Process    flake-ctl-firecracker  --help
    Should Be Equal As Integers    ${result.rc}    0