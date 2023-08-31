*** Settings ***
Library    Process
Library    OperatingSystem
Library    Collections

*** Variables ***
${APPS_DIR} =  /usr/bin
${FLAKES_DIR} =  /usr/share/flakes

*** Keywords ***
Register Podman Container
    [Arguments]    ${container}    ${name}    ${base}=None    ${target}=/    ${working_dir}=None

    ${args} =  Create List    sudo    flake-ctl    podman    register

    Append To List  ${args}    --container    ${container}
    Append To List  ${args}    --app    ${APPS_DIR}/${name}

    IF    "${base}" != "None"
        Append To List    ${args}    --base    ${base}
    END


    IF    "${working_dir}" != "None"
        Append To List    ${args}    --working-dir        
    END

    Append To List  ${args}    --target    ${target}



    ${result} =    Run Process    @{args}

    Log    ${result.stdout}
    Log    ${result.stderr}
    Should Be Equal As Strings    ${result.rc}    0
    File Should Exist    ${FLAKES_DIR}/${name}.yaml
    RETURN    ${result}


*** Keywords ***
Pull Podman Container
    [Arguments]    ${url}
    ${result} =    Run Process    flake-ctl    podman    pull    --uri    ${url}

    Log    ${result.stdout}
    Log    ${result.stderr}

    Should Be Equal As Integers    ${result.rc}    0
    # Directory Should Not Be Empty    ${HELLO_WORLD_CONTAINER_DIR}
    RETURN    ${result}


Container Should Exist
    [Arguments]    ${name}
    ${result} =    Run Process    podman    mount

    Should Contain    ${result.stdout}    ${name}
