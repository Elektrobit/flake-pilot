*** Settings ***
Library  Process

*** Test Cases ***
Smoke Test
   ${proc} =  Run Process    flake-ctl  -V
   Then Should Be Equal As Strings    ${proc.rc}    0
   Log  ${proc.stdout}

