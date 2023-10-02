#[cfg(test)]
mod tests {
    use flakes::config::itf::FlakeConfig;
    use std::path::PathBuf;

    #[test]
    fn podman_runner_create_cid() {
        assert!(
            crate::prunner::PodmanRunner::new("junkyard".to_string(), FlakeConfig::default()).get_cidfile()
                == PathBuf::from("/usr/share/flakes/cid/junkyard.cid"),
            "Wrong CID path"
        )
    }
}
