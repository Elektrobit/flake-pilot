use std::{io::Error, process::Command};

use flakes::config::itf::FlakeConfig;

/// Podman caller
///

pub(crate) struct PdSysCall {
    debug: bool,
    cfg: FlakeConfig,
}

impl PdSysCall {
    pub(crate) fn new(debug: bool) -> Self {
        PdSysCall { cfg: flakes::config::get(), debug }
    }

    /// Get config
    fn get_cfg(&self) -> &FlakeConfig {
        &self.cfg
    }

    pub(crate) fn call(&self, output: bool, args: &[&str]) -> Result<String, Error> {
        let mut cmd = Command::new("sudo");
        if let Some(user) = self.get_cfg().runtime().run_as() {
            cmd.arg("--user").arg(user.name).arg("podman");
        } else {
            cmd = Command::new("podman");
        }

        for arg in args {
            cmd.arg(arg);
        }

        if self.debug {
            log::debug!("Syscall: {:?}", cmd);
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
}
