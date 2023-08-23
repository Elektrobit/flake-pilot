use std::{process::Command, ffi::OsStr};

use serde::{Serialize, Deserialize};


#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct User<'a> {
    name: Option<&'a str>
}

impl<'a> From<&'a str> for User<'a> {
    fn from(value: &'a str) -> Self {
        Self { name: Some(value) }
    }
}

impl<'a> User<'a> {
    pub const ROOT: User<'static> = User { name: Some("root")};
    
    pub fn run<S: AsRef<OsStr>>(&self, command: S) -> Command {
        let mut c = Command::new("sudo");
        if let Some(name) = self.name {
            c.arg("--user").arg(name);
        }
        c.arg(command);
        c
    }
}
