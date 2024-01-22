use crate::prunner::PodmanRunner;
use flakes::config::itf::InstanceMode;
use std::{io::{Error, stdout, Write}, path::PathBuf};

/// Podman runtime
///
pub(crate) struct PodmanPilot {
    appdir: PathBuf,
    runner: PodmanRunner,
    debug: bool
}

impl PodmanPilot {
    /// Constructor of a new Podman Pilot instance
    pub(crate) fn new(debug: bool) -> Result<Self, Error> {
        let appdir = flakes::config::app_path()?;
        Ok(PodmanPilot {
            appdir: appdir.to_owned(),
            runner: PodmanRunner::new(appdir.file_name().unwrap().to_str().unwrap().to_string(), flakes::config::get(), debug),
            debug,
        })
    }

    /// Start Podman Pilot instance
    pub(crate) fn start(&mut self) -> Result<(), Error> {
        let jh = self.runner.cid_collect();

        if self.runner.setup_container()? && self.runner.is_running()? {
            if *self.runner.get_cfg().runtime().instance_mode() & InstanceMode::Attach == InstanceMode::Attach {
                self.runner.attach()?;
            } else {
                self.runner.exec()?;
            }
        } else {
            if self.debug {
                log::debug!("Starting a flake on {:?}", self.appdir);
            }
            self.runner.start()?;
            if *self.runner.get_cfg().runtime().instance_mode() & InstanceMode::Resume == InstanceMode::Resume {
                self.runner.exec()?;
            }
        }

        if let Err(err) = jh.join() {
            log::error!("{:?}", err);
        }

        Ok(())
    }
}
