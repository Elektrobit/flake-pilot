mod ut_rt;

/// Unit tests for v2 config
#[cfg(test)]
mod cfg_v2_ut {
    use std::path::PathBuf;

    use flakes::config::{itf::InstanceMode, pilots::fc::FirecrackerRuntimeParams};

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

    /// Test v2 path map has specific properties: exports
    #[test]
    fn test_cfg_v2_path_map_has_spec_props_exports() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                cfg.unwrap().runtime().paths().get(&PathBuf::from("/usr/bin/banana")).unwrap().exports()
                    == &PathBuf::from("/usr/bin/brown-banana"),
                "Banana should be a bit older than that"
            );
        });
    }

    /// Test v2 path map has specific properties: user
    #[test]
    fn test_cfg_v2_path_map_has_spec_props_user() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                cfg.clone().unwrap().runtime().paths().get(&PathBuf::from("/usr/bin/banana")).unwrap().run_as().is_some(),
                "Banana should have some consumer"
            );
            assert!(
                cfg.unwrap().runtime().paths().get(&PathBuf::from("/usr/bin/banana")).unwrap().run_as().unwrap().uid.is_root(),
                "Only r00t can eat bananas"
            );
        });
    }

    #[test]
    fn test_cfg_v2_path_map_has_spec_props_instance_mode() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            let banana = cfg.unwrap().clone();
            let banana = banana.runtime().paths().get(&PathBuf::from("/usr/bin/rotten-banana")).unwrap();
            assert!(
                banana.instance_mode().unwrap() & InstanceMode::Resume == InstanceMode::Resume,
                "Rotten banana should resume"
            );
        });
    }

    #[test]
    fn test_cfg_v2_path_map_has_default_path() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            let p = PathBuf::from("/usr/bin/bash");
            let banana = cfg.unwrap().clone();
            let banana = banana.runtime().paths().get(&p).unwrap();
            assert!(banana.exports() == &p, "Rotten banana should resume");
        });
    }

    #[test]
    fn test_cfg_v2_path_map_default_path_has_common_behaviour() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            let p = PathBuf::from("/usr/bin/bash");
            let banana = cfg.unwrap().clone();
            let banana = banana.runtime().paths().get(&p).unwrap();
            assert!(
                banana.instance_mode().unwrap() & InstanceMode::Resume == InstanceMode::Resume,
                "Rotten banana should be resumed"
            );
            assert!(
                banana.instance_mode().unwrap() & InstanceMode::Attach == InstanceMode::Attach,
                "Rotten banana should be still attached to a tree"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_pilot() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.unwrap().engine().pilot() == "RD2D".to_string(), "Pilot should always have RD2D!");
        });
    }

    #[test]
    fn test_cfg_v2_engine_args() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.unwrap().engine().args().is_some(), "Pilot should have instructions!");
        });
    }

    #[test]
    fn test_cfg_v2_engine_args_len() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.unwrap().engine().args().unwrap().len() == 2, "Pilot should have whole two instructions!");
        });
    }

    #[test]
    fn test_cfg_v2_engine_args_second_check() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                cfg.unwrap().engine().args().unwrap().get(1).unwrap() == "--foo=bar",
                "Pilot should know exactly where it goes!"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_exists() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.unwrap().engine().params().is_some(), "Pilot should have parameters!");
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_vm_boot_args() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(cfg.unwrap().engine().params().unwrap().get("boot_args").is_some(), "VM should have boot args!");
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_rtp_boot_args() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                FirecrackerRuntimeParams::from(cfg.unwrap().engine().params().unwrap()).boot_args().is_some(),
                "Runtime params should have boot args!"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_rtp_boot_args_len() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                FirecrackerRuntimeParams::from(cfg.unwrap().engine().params().unwrap()).boot_args().unwrap().len() == 7,
                "Runtime params should have seven params!"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_rtp_memsize() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                FirecrackerRuntimeParams::from(cfg.unwrap().engine().params().unwrap()).mem_size_mib().is_some(),
                "Runtime params should have memsize"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_rtp_memsize_check() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                FirecrackerRuntimeParams::from(cfg.unwrap().engine().params().unwrap()).mem_size_mib().unwrap() == 0x1000,
                "Runtime params should have memsize of 4096"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_rtp_vcpu() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                FirecrackerRuntimeParams::from(cfg.unwrap().engine().params().unwrap()).vcpu_count().unwrap() == 2,
                "Runtime params should have some virtual CPUs"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_rtp_cache_type() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                FirecrackerRuntimeParams::from(cfg.unwrap().engine().params().unwrap()).cache_type().unwrap() == "Writeback",
                "Runtime params should have cache type as writeback"
            );
        });
    }

    #[test]
    fn test_cfg_v2_engine_params_rtp_overlay_size() {
        ut_rt::tb("cfg-v2/all.yaml".to_string(), |cfg| {
            assert!(
                FirecrackerRuntimeParams::from(cfg.unwrap().engine().params().unwrap()).overlay_size().unwrap() == "20GiB",
                "Runtime params should have overlay size of 20 gigabytes"
            );
        });
    }
}
