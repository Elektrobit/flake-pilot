use crate::{fgc::CidGarbageCollector, pdsys::PdSysCall};
use flakes::config::itf::{FlakeConfig, InstanceMode};
use std::path::PathBuf;
use std::{fs, thread};
use std::{io::Error, thread::JoinHandle};

pub(crate) struct PodmanRunner {
    app: String,
    cfg: FlakeConfig,
    gc: CidGarbageCollector,
    cid: Option<String>,
    cidfile: Option<PathBuf>,
    pds: PdSysCall,
    debug: bool,
}

impl PodmanRunner {
    pub(crate) fn new(app: String, cfg: FlakeConfig, debug: bool) -> Self {
        PodmanRunner {
            gc: CidGarbageCollector::new(debug),
            pds: PdSysCall::new(debug),
            cid: None,
            cidfile: None,
            app,
            cfg,
            debug,
        }
    }

    /// Check if container is already running
    pub(crate) fn is_running(&self) -> Result<bool, Error> {
        let cid = self.get_cid();

        if self.debug {
            log::debug!("Checking running container for the CID: {}", cid);
        }

        for rcid in self.pds.call(true, &["ps", "--format", "{{.ID}}"])?.lines() {
            if cid.starts_with(rcid.trim()) {
                if self.debug {
                    log::debug!("Container with CID {} is already running", cid);
                }
                return Ok(true);
            }
        }

        if self.debug {
            log::debug!("Container with CID {} is not running", cid);
        }

        Ok(false)
    }

    /// Make a CID file
    pub(crate) fn get_cidfile(&mut self) -> Result<PathBuf, Error> {
        if self.cidfile.is_some() {
            return Ok(self.cidfile.to_owned().unwrap());
        }

        let mut suff = String::from("");
        for arg in std::env::args().collect::<Vec<String>>() {
            if arg.starts_with('@') {
                suff = format!("-{}", &arg.to_owned()[1..]);
                break;
            }
        }

        self.cidfile = Some(flakes::config::get_cid_store()?.join(format!("{}{}.cid", self.app.to_owned(), suff)));
        Ok(self.cidfile.to_owned().unwrap())
    }

    /// Purge bogus CID files
    pub(crate) fn cid_collect(&self) -> JoinHandle<Result<(), Error>> {
        let gc = self.gc.to_owned();
        thread::spawn(move || gc.on_all())
    }

    /// Get CID (should be initialised)
    fn get_cid(&self) -> String {
        self.cid.to_owned().unwrap()
    }

    /// Sets CID during the init
    fn set_cid(&mut self, cid: String) {
        self.cid = Some(cid.trim().to_string());
    }

    /// Generic cleanup.
    /// Its behaviour depends on the flake conditions.
    pub(crate) fn cleanup(&mut self) -> Result<(), Error> {
        if self.debug {
            log::debug!("Cleanup");
        }

        if *self.get_cfg().runtime().instance_mode() & InstanceMode::Resume != InstanceMode::Resume {
            if self.debug {
                log::debug!("Removing non-resumable CID: {:?}", self.get_cidfile());
            }

            fs::remove_file(self.get_cidfile()?)?;
        }

        Ok(())
    }

    /// Get config
    pub(crate) fn get_cfg(&self) -> &FlakeConfig {
        &self.cfg
    }

    /// Return target app from the config
    fn get_target_app(&self) -> Result<String, Error> {
        let app_path = flakes::config::app_path()?;
        if self.debug {
            log::debug!("Host path: {:?}", app_path);
        }

        let target = self.get_cfg().runtime().paths().get(&app_path);
        if target.is_none() {
            if self.debug {
                log::debug!("Unable to find specified target path by the host path. Configuration wrong?");
            }
            return Err(Error::new(std::io::ErrorKind::NotFound, "Target path not found"));
        }

        Ok(target.unwrap().exports().as_os_str().to_str().to_owned().unwrap().to_string())
    }

    /// Create a CLI arguments for preparing the container
    /// This is a part of "get_container", so likely must be just merged with it.
    ///
    /// Returns True if CID is reused (wasn't garbage collected)
    pub(crate) fn setup_container(&mut self) -> Result<bool, Error> {
        let x_cf = self.get_cidfile()?;
        if let (true, cid) = self.gc.on_cidfile(x_cf)? {
            self.set_cid(cid);

            if self.debug {
                log::debug!("Bailing out with CID: {}", self.get_cid());
            }

            return Ok(true);
        }

        let mut args: Vec<String> = vec![
            "create".to_string(),
            "--cidfile".to_string(),
            self.cidfile.to_owned().unwrap().as_os_str().to_str().unwrap().to_string(),
            "-ti".to_string(),
        ];

        if *self.get_cfg().runtime().instance_mode() & InstanceMode::Resume != InstanceMode::Resume {
            args.push("--rm".to_string());
        }

        for arg in self.get_cfg().engine().args().unwrap_or_default() {
            // Remove params that are already there
            if arg == "-ti" || arg == "--rm" {
                continue;
            }
            args.extend(arg.split(' ').filter(|x| !x.is_empty()).map(|s| s.to_string()).collect::<Vec<String>>());
        }

        // Container name or base name
        args.push(self.get_cfg().runtime().image_name().to_string());

        if *self.get_cfg().runtime().instance_mode() & InstanceMode::Resume == InstanceMode::Resume {
            // @schaefi promised to be dead by that time :)
            args.extend(vec!["sleep".to_string(), "4294967295d".to_string()]);
        } else {
            args.push(self.get_target_app()?);
        }

        // Pass the rest of the stuff to the app
        for arg in std::env::args().collect::<Vec<String>>().iter().skip(1) {
            // "@blah" are escaped by adding an extra "@" so it becomes "@@blah".
            // Here we unescape that.
            if arg.starts_with("@@") {
                args.push(arg[1..].to_string());
            } else if !arg.starts_with('@') {
                args.push(arg.to_string());
            }
        }

        self.set_cid(self.pds.call(true, &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?);

        if self.debug {
            log::debug!("Processing with CID: {}", self.get_cid());
        }

        Ok(false)
    }

    /// Run the container with the constructed calls.
    /// Internal method
    fn _run(&mut self, output: bool, args: Vec<String>) -> Result<(String, String), Error> {
        let stdout = self.pds.call(output, &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?;
        self.cleanup()?;
        Ok((stdout, "".to_string()))
    }

    /// Launch a container
    pub(crate) fn start(&mut self) -> Result<(String, String), Error> {
        if self.debug {
            log::debug!("Starting container {}", self.get_cid()[..0xc].to_string());
        }

        self._run(true, vec!["start".to_string(), "--attach".to_string(), self.get_cid()])
    }

    /// Attach to a container
    pub(crate) fn attach(&mut self) -> Result<(String, String), Error> {
        if self.debug {
            log::debug!("Attaching to the container {}", self.get_cid()[..0xc].to_string());
        }

        self._run(true, vec!["attach".to_string(), self.get_cid(), self.get_target_app()?])
    }

    /// Exec a container
    pub(crate) fn exec(&mut self) -> Result<(String, String), Error> {
        if self.debug {
            log::debug!("Resuming container interactively {}", self.get_cid()[..0xc].to_string());
        }

        self._run(
            false,
            vec!["exec".to_string(), "--interactive".to_string(), "--tty".to_string(), self.get_cid(), self.get_target_app()?],
        )
    }
}
