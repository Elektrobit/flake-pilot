mod ut_rt;

/// Unit tests for v2 config
#[cfg(test)]
mod cfg_v2_ut {
    use std::path::PathBuf;

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

    /// Test v2 path map
    #[test]
    fn test_cfg_v2_path_map_present() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(!cfg.clone().unwrap().runtime().paths().is_empty(), "Path map should not be empty");
            assert!(cfg.unwrap().runtime().paths().len() == 4, "Path map should have four elements");
        });
    }

    /// Test v2 path map has properties
    #[test]
    fn test_cfg_v2_path_map_has_props() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                cfg.unwrap().runtime().paths().get(&PathBuf::from("/usr/bin/banana")).is_some(),
                "Banana should have some properties!"
            );
        });
    }
}
