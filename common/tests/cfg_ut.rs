#[cfg(test)]
mod tests {
    use core::panic;
    use flakes::config::{cfgparse::FlakeCfgParser, itf::FlakeConfig};
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
