use std::{fs, io::Error, ops::Deref, path::PathBuf, process::Command, vec};

use flakes::config::{
    itf::{FlakeConfig, InstanceMode},
    CID_DIR,
};

use crate::datasync::DataSync;

pub(crate) struct PodmanRunner {
    datasync: DataSync,
    app: String,
    cfg: FlakeConfig,
    cid: Option<String>,
    cidfile: Option<PathBuf>,
}

impl PodmanRunner {
    pub(crate) fn new(app: String, cfg: FlakeConfig) -> Self {
        PodmanRunner { datasync: DataSync {}, app, cfg, cid: None, cidfile: None }
    }

    /// Make a CID file
    pub(crate) fn get_cidfile(&mut self) -> PathBuf {
        if self.cidfile.is_some() {
            return self.cidfile.to_owned().unwrap();
        }

        let mut suff = String::from("");
        for arg in std::env::args().collect::<Vec<String>>() {
            if arg.starts_with('@') {
                suff = format!("-{}", &arg.to_owned()[1..]);
                break;
            }
        }

        self.cidfile = Some(CID_DIR.join(format!("{}{}.cid", self.app.to_owned(), suff)));
        self.cidfile.to_owned().unwrap()
    }

    fn get_cid(&self) -> String {
        self.cid.to_owned().unwrap()
    }

    /// Garbage collect CID.
    ///
    /// Check if container exists according to the specified
    /// container_cid_file. Garbage cleanup the container_cid_file
    /// if no longer present. Return a true value if the container
    /// exists, in any other case return false.
    pub(crate) fn gc_cidfile(&self, cidfile: PathBuf) -> Result<(bool, String), Error> {
        if !cidfile.exists() {
            return Ok((false, "".to_string()));
        }

        let cid = &fs::read_to_string(&cidfile)?;

        match self.call(false, &["container", "exists", cid.trim()]) {
            Ok(_) => {
                log::debug!("Container with CID {:?} exists", cidfile);
                Ok((true, cid.to_string()))
            }
            Err(_) => {
                fs::remove_file(&cidfile)?;
                log::debug!("Container with CID {:?} does not exist, removing CID", cidfile);
                Ok((false, "".to_string()))
            }
        }
    }

    /// Get config
    fn get_cfg(&self) -> &FlakeConfig {
        &self.cfg
    }

    fn call(&self, output: bool, args: &[&str]) -> Result<String, Error> {
        let mut cmd = Command::new("sudo");
        if let Some(user) = self.get_cfg().runtime().run_as() {
            cmd.arg("--user").arg(user.name).arg("podman");
        } else {
            cmd = Command::new("podman");
        }

        for arg in args {
            cmd.arg(arg);
        }

        log::debug!("Syscall: {:?}", cmd);

        if output {
            match cmd.output() {
                Ok(out) => {
                    return Ok(String::from_utf8(out.stdout).unwrap_or_default());
                }
                Err(out) => {
                    return Err(out);
                }
            }
        } else {
            match cmd.status() {
                Ok(st) => {
                    if !st.success() {
                        return Err(Error::new(std::io::ErrorKind::InvalidData, "Call error"));
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
        Ok("".to_string())
    }

    /// Create a CLI arguments for preparing the container
    /// This is a part of "get_container", so likely must be just merged with it
    fn setup_container(&mut self) -> Result<(), Error> {
        let mut args: Vec<String> = vec![];

        self.get_cidfile();
        if let (true, cid) = self.gc_cidfile(self.cidfile.clone().unwrap())? {
            self.cid = Some(cid);
            return Ok(());
        }

        let app_path = flakes::config::app_path()?;
        log::debug!("Host path: {:?}", app_path);

        let target = self.cfg.runtime().paths().get(&app_path);
        if target.is_none() {
            log::debug!("Unable to find specified target path by the host path. Configuration wrong?");
            return Err(Error::new(std::io::ErrorKind::NotFound, "Target path not found"));
        }

        self.datasync.check_cid_dir()?;
        args.extend(vec![
            "create".to_string(),
            "--cidfile".to_string(),
            self.cidfile.to_owned().unwrap().as_os_str().to_str().unwrap().to_string(),
        ]);

        let resume = *self.cfg.runtime().instance_mode() & InstanceMode::Resume == InstanceMode::Resume;
        if resume {
            args.push("-ti".to_string());
        } else {
            args.push("--rm".to_string());
            args.push("-ti".to_string());
        }

        for arg in self.cfg.engine().args().unwrap_or(vec![]) {
            if arg == "-ti" || arg == "--rm" {
                continue;
            }
            args.extend(arg.split(' ').filter(|x| !x.is_empty()).map(|s| s.to_string()).collect::<Vec<String>>());
        }

        // Container name or base name
        args.push(self.cfg.runtime().image_name().to_string());

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

        self.cid = Some(self.call(true, &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?);
        log::debug!("CID: {}", self.get_cid());

        Ok(())
    }

    /// Launch a container
    pub(crate) fn start(&mut self) -> Result<(PathBuf, String), Error> {
        self.setup_container()?;

        // Construct args for launching an instance
        let mut args: Vec<String> = vec!["start".to_string()];
        let resume = *self.cfg.runtime().instance_mode() & InstanceMode::Resume == InstanceMode::Resume;

        if resume {
            // mute STDOUT
        } else {
            args.push("--attach".to_string());
        }
        args.push(self.cid.to_owned().unwrap());
        let out = self.call(true, &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?;
        log::info!("{}", out);

        Ok((self.get_cidfile(), self.get_cid()))
    }

    pub(crate) fn attach(&mut self) -> Result<(PathBuf, String), Error> {
        Ok((self.get_cidfile(), self.get_cid()))
    }
}
