use std::{fs, io::Error, path::PathBuf, process::Command};

use flakes::config::{
    itf::{FlakeConfig, InstanceMode},
    CID_DIR,
};

use crate::datasync::DataSync;

pub(crate) struct PodmanRunner {
    datasync: DataSync,
    app: String,
    cfg: FlakeConfig,
}

impl PodmanRunner {
    pub(crate) fn new(app: String, cfg: FlakeConfig) -> Self {
        PodmanRunner { datasync: DataSync {}, app, cfg }
    }

    /// Make a CID file
    pub(crate) fn get_cid(&self) -> PathBuf {
        let mut suff = String::from("");
        for arg in std::env::args().collect::<Vec<String>>() {
            if arg.starts_with('@') {
                suff = format!("-{}", &arg.to_owned()[1..]);
                break;
            }
        }

        CID_DIR.join(format!("{}{}.cid", self.app.to_owned(), suff))
    }

    /// Garbage collect CID.
    ///
    /// Check if container exists according to the specified
    /// container_cid_file. Garbage cleanup the container_cid_file
    /// if no longer present. Return a true value if the container
    /// exists, in any other case return false.
    pub(crate) fn gc_cid(&self, cid: PathBuf) -> Result<bool, Error> {
        if !cid.exists() {
            return Ok(false);
        }

        match self.call(false, &["container", "exists", &fs::read_to_string(&cid)?.trim()]) {
            Ok(_) => {
                log::debug!("Container with CID {:?} exists", cid);
                Ok(true)
            }
            Err(_) => {
                fs::remove_file(&cid)?;
                log::debug!("Container with CID {:?} does not exist, removing CID", cid);
                Ok(false)
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
    pub(crate) fn get_container(&self) -> Result<(PathBuf, String), Error> {
        let mut args: Vec<String> = vec![];

        let cidfile = self.get_cid();
        if self.gc_cid(cidfile.to_owned())? {
            return Ok((cidfile, "".to_string()));
        }

        let app_path = flakes::config::app_path()?;
        log::debug!("Host path: {:?}", app_path);

        let target = self.cfg.runtime().paths().get(&app_path);
        if target.is_none() {
            log::debug!("Unable to find specified target path by the host path. Configuration wrong?");
            return Err(Error::new(std::io::ErrorKind::NotFound, "Target path not found"));
        }

        self.datasync.check_cid_dir()?;
        args.extend(vec!["create".to_string(), "--cidfile".to_string(), cidfile.as_os_str().to_str().unwrap().to_string()]);

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

        let out = self.call(true, &args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())?;
        log::debug!("CID: {}", out);

        Ok((PathBuf::from(""), "".to_string()))
    }
}
