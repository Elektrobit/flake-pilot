*** Settings ***
Library    Process
*** Test Cases ***
Firecracker is installed
    ${result} =  Run Process  firecracker  --version
    Log  ${result.stdout}
    Log  ${result.stderr}
    Should Be Equal As Integers    ${result.rc}    0
