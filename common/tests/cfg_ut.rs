#[cfg(test)]
mod cfg_v1_ut {
    use core::panic;
    use flakes::config::{
        self,
        cfgparse::FlakeCfgParser,
        itf::{self, FlakeConfig, InstanceMode},
    };
    use std::{env, path::PathBuf};

    /// Setup the test
    fn setup(cfg_path: String) -> Option<FlakeConfig> {
        if let Ok(parser) = FlakeCfgParser::new(env::current_dir().unwrap().join("tests").join("data").join(&cfg_path), vec![]) {
            return parser.parse();
        }

        None
    }

    /// Run a test bundle
    fn tb<T>(cfg_path: String, probe: T) -> ()
    where
        T: Fn(Option<FlakeConfig>) -> () + panic::UnwindSafe,
    {
        probe(setup(cfg_path));
    }

    /// Test Firecracker configuration v1 overall parse
    #[test]
    fn test_cfg_v1_fc_overall_parse() {
        tb("cfg-v1/firecracker.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v1 for firecracker should not be None");
            assert!(cfg.unwrap().version() == 1, "Version should be 1");
        });
    }

    /// Test bogus configuration v42 overall parse
    #[test]
    fn test_cfg_v42_overall_parse() {
        tb("bogus.yaml".to_string(), |cfg| {
            assert!(cfg.is_none(), "FlakeConfig v42 should be None and must be unsupported");
        });
    }

    /// Test podman configuration v1 overall parse
    #[test]
    fn test_cfg_v1_pdm_overall_parse() {
        tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v1 for podman should not be None");
            assert!(cfg.unwrap().version() == 1, "Version should be 1");
        });
    }

    /// Test OCI name
    #[test]
    fn test_cfg_v1_pdm_name() {
        tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().image_name() == "banana");
        });
    }

    /// Test OCI target path
    #[test]
    fn test_cfg_v1_pdm_exported_app_path() {
        tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().paths().exported_app_path() == &PathBuf::from("/banana/in/the/container"));
        });
    }

    /// Test OCI target path
    #[test]
    fn test_cfg_v1_pdm_registered_app_path() {
        tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().paths().registered_app_path() == &PathBuf::from("/usr/bin/banana"));
        });
    }

    /// Test OCI base layer
    #[test]
    fn test_cfg_v1_pdm_base_layer() {
        tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().base_layer().is_some(), "Base layer should be defined");
            assert!(cfg.runtime().base_layer().unwrap() == "cobol_rules");
        });
    }

    /// Test OCI additional layers
    #[test]
    fn test_cfg_v1_pdm_layers() {
        tb("cfg-v1/podman.yaml".to_string(), |cfg| {
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
        tb("cfg-v1/podman.yaml".to_string(), |cfg| {
            let cfg = cfg.unwrap();
            assert!(cfg.runtime().run_as().is_some(), "User should be defined");
            assert!(cfg.runtime().run_as().unwrap().uid.is_root(), "User should be root");
            assert!(cfg.runtime().run_as().unwrap().name == "root", "User should be root");
        });
    }

    /* ------- V2 ------- */
    /// Test v2 overall parse
    #[test]
    fn test_cfg_v2_overall_parse() {
        tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v2 should not be None");
            assert!(cfg.unwrap().version() == 2, "Version should be 2");
        });
    }
}
