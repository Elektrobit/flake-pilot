*** Settings ***
Library     Process
Library     OperatingSystem


*** Variables ***
${NON_EXISTING_CONTAINER_URL} =         todo.example.com/non/exisitng/container

${HELLO_WORLD_CONTAINER_URL} =          todo.examplepodman.com/hello/world
${HELLO_WORLD_CONTAINER_DIR} =          todo.examplepodman.com/hello/world

${HELLO_WORLD_ADDON_CONTAINER_URL} =    todo.examplepodman.com/addon


*** Test Cases ***
Pull a Container
    ${result} =    Pull Container    ${HELLO_WORLD_CONTAINER_URL}

    [Teardown]    Remove Directory    ${HELLO_WORLD_CONTAINER_DIR}

Pull a non-exisitng Container
    Run Keyword And Expect Error    125 != 0    Pull Container    ${NON_EXISTING_CONTAINER_URL}

Regist a non-exisitng Container
    Run Keyword And Expect Error    2 != 0    Register Container    ThisContainerDoesNotExist

Register a Container
    Pull Container    ${HELLO_WORLD_CONTAINER_URL}
    And Register Container    hello_world
    Then Container should Exist    hello_world

Register a Base Container
    Pull Container    ${HELLO_WORLD_CONTAINER_URL}
    And Pull Container    ${HELLO_WORLD_ADDON_CONTAINER_URL}
    And Register Container    hello_world_addon  hello_world


*** Keywords ***
Pull Container
    [Arguments]    ${url}
    ${result} =    Run Process    flake-ctl    podman    pull    --uri    ${url}    stdout=PIPE

    Should Be Equal As Strings    ${result.rc}    0
    Directory Should Not Be Empty    ${HELLO_WORLD_CONTAINER_DIR}
    RETURN    ${result}


Register Container
    [Arguments]    ${name}    ${base}=None

    IF    ${base} != "None"
        ${result} =    Run Process    flake-ctl    podman    register    --container    ${name}    stdout=PIPE
    ELSE
        ${result} =    Run Process
        ...    flake-ctl
        ...    podman
        ...    register
        ...    --container
        ...    ${name}
        ...    --base
        ...    ${base}
        ...    stdout=PIPE
    END

    Should Be Equal As Strings    ${result.rc}    0
    File Should Exist    /usr/share/flakes/${name}.yaml
    RETURN    ${result}


Container Should Exist
    [Arguments]    ${name}
    ${result} =    Run Process    podman    mount    stdout=PIPE

    Should Contain    ${result.stdout}    ${name}
