#[cfg(test)]
mod tests {
    use flakes::config::itf::FlakeConfig;
    use std::path::PathBuf;

    #[test]
    fn podman_runner_create_cid() {
        /*
        match crate::prunner::PodmanRunner::new("junkyard".to_string(), FlakeConfig::default(), false).get_cidfile() {
            Ok(cf) => assert!(cf == PathBuf::from("/usr/share/flakes/cid/junkyard.cid"), "Wrong CID path"),
            Err(err) => assert!(false, "Error: {}", err),
        }
        */
    }
}
