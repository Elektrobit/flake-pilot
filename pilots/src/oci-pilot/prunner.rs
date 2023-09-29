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
            return Ok(true);
        }

        match self.call(false, &["container", "exists", &fs::read_to_string(&cid)?]) {
            Ok(_) => Ok(true),
            Err(_) => {
                fs::remove_file(cid)?;
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

    /// Create a container
    pub(crate) fn create_container(&self) -> Result<(PathBuf, String), Error> {
        let cid = self.get_cid();
        if !self.gc_cid(cid.to_owned())? {
            return Ok((cid, "".to_string()));
        }

        println!("Getting host path: {:?}", flakes::config::app_path()?);

        let target = self.cfg.runtime().paths().get(&flakes::config::app_path()?);
        if target.is_none() {
            return Err(Error::new(std::io::ErrorKind::NotFound, "Target path not found"));
        }

        self.datasync.check_cid_dir()?;

        let mut args: Vec<String> =
            vec!["create".to_string(), "--cidfile".to_string(), cid.as_os_str().to_str().unwrap().to_string()];

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
            args.push(arg);
        }

        // Container name or base name
        args.push(self.cfg.runtime().image_name().to_string());

        if resume {
            args.push("sleep".to_string());
            args.push("4294967295d".to_string());
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

        println!("CALL:\n{:?}", args);
        println!("CFG:\n{:?}", self.cfg.runtime().paths());

        //self.call(&["create", "--cidfile", cid.as_os_str().to_str().unwrap()])?;

        Ok((cid, "".to_string()))
    }
}
