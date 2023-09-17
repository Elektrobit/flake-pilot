#[cfg(test)]
mod tests {
    use core::panic;
    use flakes::config::{cfgparse::FlakeCfgParser, itf::FlakeConfig};
    use std::path::PathBuf;

    /// Setup the test
    fn setup(cfg_path: String) -> Option<FlakeConfig> {
        FlakeCfgParser::new(PathBuf::from(cfg_path)).parse()
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
        tb("data/cfg-v1/firecracker.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v1 for firecracker should not be None");
            assert!(cfg.unwrap().version() == 1, "Version should be 1");
        });
    }

    /// Test podman configuration v1 overall parse
    #[test]
    fn test_cfg_v1_pdm_overall_parse() {
        tb("data/cfg-v1/podman.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v1 for podman should not be None");
            assert!(cfg.unwrap().version() == 1, "Version should be 1");
        });
    }
}
