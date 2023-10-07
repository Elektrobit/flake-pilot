use crate::prunner::PodmanRunner;
use flakes::config::itf::InstanceMode;
use std::{io::Error, path::PathBuf};

/// Podman runtime
///
pub(crate) struct PodmanPilot {
    appdir: PathBuf,
    runner: PodmanRunner,
    debug: bool,
    stdout: String,
    stderr: String,
}

impl PodmanPilot {
    /// Constructor of a new Podman Pilot instance
    pub(crate) fn new(debug: bool) -> Result<Self, Error> {
        let appdir = flakes::config::app_path()?;
        Ok(PodmanPilot {
            appdir: appdir.to_owned(),
            runner: PodmanRunner::new(appdir.file_name().unwrap().to_str().unwrap().to_string(), flakes::config::get(), debug),
            stdout: "".to_string(),
            stderr: "".to_string(),
            debug,
        })
    }

    /// Start Podman Pilot instance
    pub(crate) fn start(&mut self) -> Result<(), Error> {
        if self.runner.setup_container()? && self.runner.is_running()? {
            if *self.runner.get_cfg().runtime().instance_mode() & InstanceMode::Attach == InstanceMode::Attach {
                (self.stdout, self.stderr) = self.runner.attach()?;
            } else {
                (self.stdout, self.stderr) = self.runner.exec()?;
            }
        } else {
            if self.debug {
                log::debug!("Starting a flake on {:?}", self.appdir);
            }
            (self.stdout, self.stderr) = self.runner.start()?;
            if *self.runner.get_cfg().runtime().instance_mode() & InstanceMode::Resume == InstanceMode::Resume {
                (self.stdout, self.stderr) = self.runner.exec()?;
            }
        }

        // Print out the results
        if !self.stdout.is_empty() {
            println!("{}", self.stdout);
        }

        if !self.stderr.is_empty() {
            log::error!("{}", self.stderr);
        }

        Ok(())
    }

    fn is_image_exists(&self, name: &str) -> Result<bool, Error> {
        Ok(false)
    }

    /// Mount container. Returns the mount point, if succeeded
    fn mount(&self, as_image: bool) -> Result<PathBuf, Error> {
        Ok(PathBuf::from("/mount/point"))
    }

    /// Umount container.
    fn umount(&self, mountpoint: PathBuf, as_image: bool) -> Result<(), Error> {
        Ok(())
    }
}
