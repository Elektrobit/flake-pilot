#[cfg(test)]
mod tests {
    use core::panic;
    use flakes::config::{cfgparse::FlakeCfgParser, itf::FlakeConfig};
    use std::env;

    /// Setup the test
    fn setup(cfg_path: String) -> Option<FlakeConfig> {
        FlakeCfgParser::new(env::current_dir().unwrap().join("tests").join("data").join(&cfg_path)).parse()
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
