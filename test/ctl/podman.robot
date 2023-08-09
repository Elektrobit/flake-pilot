*** Settings ***
Library     Process
Library     OperatingSystem

Test Teardown  Teardown


*** Test Cases ***
Pull a Container
    ${result} =    Pull Container    docker.io/amazon/aws-cli

    # [Teardown]    Remove Directory    ${HELLO_WORLD_CONTAINER_DIR}

Pull a non-existing Container
    Run Keyword And Expect Error    125 != 0    Pull Container    this.url.does.not.exist

Register a non-existing Container
    Run Keyword And Expect Error    *    Register Container    ThisContainerDoesNotExist    Nothing

Register the same Flake twice
    Pull Container    docker.io/amazon/aws-cli
    And Register Container    amazon/aws-cli    podman-test
    Run Keyword And Expect Error    *    Register Container    amazon/aws-cli    podman-test

Register a Container
    Pull Container    docker.io/amazon/aws-cli
    And Register Container    amazon/aws-cli    podman-test
    [Teardown]  Run Process    sudo  rm  -r  /usr/bin/podman-test

Register a Base Container
    Pull Container    registry.opensuse.org/home/marcus.schaefer/delta_containers/containers_suse/joe
    And Register Base Container
    ...    joe
    ...    joe
    ...    registry.opensuse.org/home/marcus.schaefer/delta_containers/containers_suse/basesystem
    [Teardown]  Run Process    sudo  rm  -r  /usr/bin/joe


*** Keywords ***
Pull Container
    [Arguments]    ${url}
    ${result} =    Run Process    flake-ctl    podman    pull    --uri    ${url}

    Log    ${result.stdout}
    Log    ${result.stderr}

    Should Be Equal As Strings    ${result.rc}    0
    # Directory Should Not Be Empty    ${HELLO_WORLD_CONTAINER_DIR}
    RETURN    ${result}

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

Register Base Container
    [Arguments]    ${container}    ${name}    ${base}
    ${result} =    Run Process    sudo
    ...    flake-ctl    podman    register
    ...    --container    ${container}
    ...    --app    /usr/bin/${name}
    ...    --target    /
    ...    --base    ${base}

    Log    ${result.stdout}
    Log    ${result.stderr}
    Should Be Equal As Strings    ${result.rc}    0
    File Should Exist    /usr/share/flakes/${name}.yaml
    RETURN    ${result}

Container Should Exist
    [Arguments]    ${name}
    ${result} =    Run Process    podman    mount

    Should Contain    ${result.stdout}    ${name}

Teardown
    Run Process    sudo  rm  -r  /usr/share/flakes
    Run Process    podman  prune  -a  -f
