mod ut_rt;

/// Unit tests for v2 config
#[cfg(test)]
mod cfg_v2_ut {
    use super::ut_rt;

    /// Test v2 overall parse
    #[test]
    fn test_cfg_v2_overall_parse() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.is_some(), "FlakeConfig v2 should not be None");
            assert!(cfg.unwrap().version() == 2, "Version should be 2");
        });
    }

    /// Test v2 overall parse
    #[test]
    fn test_cfg_v2_runtime_name() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.unwrap().runtime().image_name() == "darth vader", "The name should be defined");
        });
    }
}
