mod ut_rt;

#[cfg(test)]
mod cfg_v1_ut {
    use super::ut_rt;
    use flakes::config::itf::InstanceMode;
    use std::path::PathBuf;

    /// Test Firecracker configuration v1 overall parse
    #[test]
    fn test_cfg_v1_fc_overall_parse() {
        ut_rt::tb("cfg-v1/firecracker.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v1 for firecracker should not be None");
            assert!(cfg.unwrap().version() == 1, "Version should be 1");
        });
    }

    /// Test bogus configuration v42 overall parse
    #[test]
    fn test_cfg_v42_overall_parse() {
        ut_rt::tb("bogus.yaml".to_string(), |cfg| {
            assert!(cfg.is_none(), "FlakeConfig v42 should be None and must be unsupported");
        });
    }

    /// Test podman configuration v1 overall parse
    #[test]
    fn test_cfg_v1_pdm_overall_parse() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v1 for podman should not be None");
            assert!(cfg.unwrap().version() == 1, "Version should be 1");
        });
    }

    /// Test OCI name
    #[test]
    fn test_cfg_v1_pdm_name() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().image_name() == "banana");
        });
    }

    /// Test OCI target path
    #[test]
    fn test_cfg_v1_pdm_exported_app_path() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().paths().exported_app_path() == &PathBuf::from("/banana/in/the/container"));
        });
    }

    /// Test OCI target path
    #[test]
    fn test_cfg_v1_pdm_registered_app_path() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().paths().registered_app_path() == &PathBuf::from("/usr/bin/banana"));
        });
    }

    /// Test OCI base layer
    #[test]
    fn test_cfg_v1_pdm_base_layer() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().base_layer().is_some(), "Base layer should be defined");
            assert!(cfg.runtime().base_layer().unwrap() == "cobol_rules");
        });
    }

    /// Test OCI additional layers
    #[test]
    fn test_cfg_v1_pdm_layers() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().layers().is_some(), "There should be more than one additional layers");
            assert!(cfg.runtime().layers().unwrap().len() == 2);
            assert!(cfg.runtime().layers().unwrap().get(0).is_some(), "First layer should be defined");
            assert!(cfg.runtime().layers().unwrap().get(1).is_some(), "Second layer should be defined");
            assert!(cfg.runtime().layers().unwrap().get(0).unwrap() == "fortran_for_web");
            assert!(cfg.runtime().layers().unwrap().get(1).unwrap() == "prolog_for_productivity");
        });
    }

    /// Test OCI container needs to be run as user root (UID 0)
    #[test]
    fn test_cfg_v1_pdm_run_as() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().run_as().is_some(), "User should be defined");
            assert!(cfg.runtime().run_as().unwrap().uid.is_root(), "User should be root");
            assert!(cfg.runtime().run_as().unwrap().name == "root", "User should be root");
        });
    }

    /// Test OCI should be resumed and attached
    #[test]
    fn test_cfg_v1_pdm_mode_flags() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!((*cfg.runtime().instance_mode() & InstanceMode::Attach) == InstanceMode::Attach, "Should have attach flag");
            assert!((*cfg.runtime().instance_mode() & InstanceMode::Resume) == InstanceMode::Resume, "Should have resume flag");
        });
    }

    /// Test OCI target podman args
    #[test]
    fn test_cfg_v1_pdm_args() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.engine().args().is_some(), "Podman should run with some parameters");
        });
    }

    /// Test OCI target podman args examination
    #[test]
    fn test_cfg_v1_pdm_args_exm() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let args = cfg.unwrap().engine().args().unwrap();
            assert!(args.len() == 3, "Podman should have three params");
            assert!(args == ["--storage-opt size=10G", "--rm", "-ti"], "Podman should have parameters in a certain order");
        });
    }

    /// Test OCI includes
    #[test]
    fn test_cfg_v1_pdm_static() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.static_data().get_bundles().is_some(), "Podman wants to include something");
        });
    }

    /// Test OCI includes contains a specific archive
    #[test]
    fn test_cfg_v1_pdm_static_data() {
        ut_rt::tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(
                cfg.static_data().get_bundles().unwrap() == ["irq-dropout.tar.gz"],
                "Podman needs to drop some IRQs during the high pressure"
            );
        });
    }

    /* ------- V2 ------- */
    /// Test v2 overall parse
    #[test]
    fn test_cfg_v2_overall_parse() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v2 should not be None");
            assert!(cfg.unwrap().version() == 2, "Version should be 2");
        });
    }
}
