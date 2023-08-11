*** Settings ***
Library     Process
Library     OperatingSystem

Test Teardown  Delete all Containers and Flakes


Resource    ../common/ctl.robot

*** Test Cases ***
Pull a Container
    ${result} =    Pull Podman Container    docker.io/amazon/aws-cli

    # [Teardown]    Remove Directory    ${HELLO_WORLD_CONTAINER_DIR}

Pull a non-existing Container
    Run Keyword And Expect Error    125 != 0    Pull Podman Container    this.url.does.not.exist

Register a non-existing Container
    Run Keyword And Expect Error    *    Register Podman Container    ThisContainerDoesNotExist    Nothing

Register the same Flake twice
    Pull Podman Container    docker.io/amazon/aws-cli
    And Register Podman Container    amazon/aws-cli    podman-test
    Run Keyword And Expect Error    *    Register Podman Container    amazon/aws-cli    podman-test
    [Teardown]  Run Process    sudo  rm  -r  /usr/bin/podman-test

Register a Container
    Pull Podman Container    docker.io/amazon/aws-cli
    And Register Podman Container    amazon/aws-cli    podman-test
    [Teardown]  Run Process    sudo  rm  -r  /usr/bin/podman-test

Register a Container with Base
    Pull Podman Container    registry.opensuse.org/home/marcus.schaefer/delta_containers/containers_suse/joe
    And Register Podman Container
    ...    joe
    ...    joe
    ...    base=registry.opensuse.org/home/marcus.schaefer/delta_containers/containers_suse/basesystem
    [Teardown]  Run Process    sudo  rm  -r  /usr/bin/joe

*** Keywords ***
Delete all Containers and Flakes
    Run Process    sudo  rm  -r  /usr/share/flakes
    Run Process    podman  prune  -a  -f
