use crate::{datasync::DataSync, fgc::CidGarbageCollector, pdsys::PdSysCall};
use flakes::config::itf::{FlakeConfig, InstanceMode};
use std::{borrow::BorrowMut, fs, io::Error, path::PathBuf, vec};

pub(crate) struct PodmanRunner {
    datasync: DataSync,
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
            datasync: DataSync {},
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

        for rcid in self.pds.call(true, &vec!["ps", "--format", "{{.ID}}"])?.lines() {
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

    fn get_cid(&self) -> String {
        self.cid.to_owned().unwrap()
    }

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

    /// Create a CLI arguments for preparing the container
    /// This is a part of "get_container", so likely must be just merged with it.
    ///
    /// Returns True if CID is reused (wasn't garbage collected)
    pub(crate) fn setup_container(&mut self) -> Result<bool, Error> {
        let mut args: Vec<String> = vec![];

        let x_cf = self.get_cidfile()?;
        if let (true, cid) = self.gc.on_cidfile(x_cf)? {
            self.set_cid(cid);

            if self.debug {
                log::debug!("Bailing out with CID: {}", self.get_cid());
            }

            return Ok(true);
        }

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

        args.extend(vec![
            "create".to_string(),
            "--cidfile".to_string(),
            self.cidfile.to_owned().unwrap().as_os_str().to_str().unwrap().to_string(),
        ]);

        let resume = *self.get_cfg().runtime().instance_mode() & InstanceMode::Resume == InstanceMode::Resume;
        if resume {
            args.push("-ti".to_string());
        } else {
            args.push("--rm".to_string());
            args.push("-ti".to_string());
        }

        for arg in self.get_cfg().engine().args().unwrap_or(vec![]) {
            if arg == "-ti" || arg == "--rm" {
                continue;
            }
            args.extend(arg.split(' ').filter(|x| !x.is_empty()).map(|s| s.to_string()).collect::<Vec<String>>());
        }

        // Container name or base name
        args.push(self.get_cfg().runtime().image_name().to_string());

        if resume {
            args.push("sleep".to_string());
            args.push("4294967295d".to_string()); // @schaefi promised to be dead by that time :)
        } else {
            args.push(target.unwrap().exports().as_os_str().to_str().to_owned().unwrap().to_string());
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

    /// Launch a container
    pub(crate) fn start(&mut self) -> Result<(String, String), Error> {
        // Construct args for launching an instance
        let mut args: Vec<String> = vec!["start".to_string()];

        args.push("--attach".to_string());
        args.push(self.get_cid());

        if self.debug {
            log::debug!("Using container {}", self.get_cid()[..0xc].to_string());
        }

        let stdout = self.pds.call(true, &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?;
        self.cleanup()?;

        Ok((stdout, "".to_string()))
    }

    pub(crate) fn attach(&mut self) -> Result<(String, String), Error> {
        Ok(("".to_string(), "attaching to a container is not yet implemented".to_string()))
    }

    pub(crate) fn exec(&mut self) -> Result<(String, String), Error> {
        Ok(("".to_string(), "resumable containers are not yet supported".to_string()))
    }
}
