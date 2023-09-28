#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use flakes::config::itf::FlakeConfig;

    use crate::podman;

    #[test]
    fn podman_runner_create_cid() {
        assert!(
            podman::PodmanRunner::new("junkyard".to_string(), FlakeConfig::default()).create_cid()
                == PathBuf::from("/usr/share/flakes/cid/junkyard.cid"),
            "Wrong CID path"
        )
    }
}
