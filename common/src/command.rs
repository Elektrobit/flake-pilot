use std::{fmt::{Display, Write}, process::{Command, Output, CommandArgs}, ffi::OsStr};

use thiserror::Error;

pub trait CommandExtTrait {
    /// Execute this command and return:
    /// 
    /// 1. An IO Error if the command could not be run
    /// 2. An Execution Error if the Command was not successfull
    /// 3. The [Output] of the Command if the command was executed successfully
    /// 
    /// Attaches all args to the resulting error
    /// 
    /// If a termination with a non 0 exit status is considered succesful
    /// this method should not be used.
    fn perform(&mut self) -> Result<std::process::Output, CommandError>;
}

impl CommandExtTrait for Command {
    fn perform(&mut self) -> Result<std::process::Output, CommandError> {
        handle_output(self.output(), self.get_args())
    }
}

pub fn handle_output(maybe_output: Result<Output, std::io::Error>, args: CommandArgs) -> Result<std::process::Output, CommandError> {
    let out = maybe_output.map_err(ProcessError::IO);

    let error: ProcessError = match out {
        Ok(output) => {
            if output.status.success() {
                return Ok(output);
            } else {
                output.into()
            }
        }
        Err(err) => err,
    };

    Err(CommandError {
        base: error,
        args: args
            .flat_map(OsStr::to_str)
            .map(ToOwned::to_owned)
            .collect(),
    })
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    // The Command terminated correctly but with unwanted results (e.g. wrong return code)
    #[error("The process failed with status {}", .0.status)]
    ExecutionError(std::process::Output),
}

impl From<std::process::Output> for ProcessError {
    fn from(value: std::process::Output) -> Self {
        Self::ExecutionError(value)
    }
}

#[derive(Debug, Error)]
pub struct CommandError {
    pub base: ProcessError,
    pub args: Vec<String>,
}

impl CommandError {
    pub fn new(base: ProcessError) -> Self {
        Self { args: Vec::new(), base }
    }

    pub fn with(&mut self, arg: String) -> &mut Self {
        self.args.push(arg);
        self
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for arg in self.args.iter() {
            f.write_str(arg)?;
            f.write_char(' ')?;
        }
        std::fmt::Display::fmt(&self.base, f)
    }
}

impl From<std::process::Output> for CommandError {
    fn from(value: std::process::Output) -> Self {
        Self { base: value.into(), args: Default::default() }
    }
}

impl From<ProcessError> for CommandError {
    fn from(value: ProcessError) -> Self {
        Self { base: value, args: Default::default() }
    }
}
