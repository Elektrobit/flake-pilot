use std::{
    error::Error,
    ffi::OsStr,
    fmt::{Debug, Display, Write},
    process::{Command, ExitCode, Output, Termination},
};

#[derive(Debug)]
pub enum FlakeError {
    /// The pilot tried to run a sub command and failed
    CommandError(CommandError),
    /// There was an error in an IO operation
    IO(std::io::Error),
    /// This flake is already running
    AlreadyRunning,
}

impl Display for FlakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlakeError::CommandError(err) => std::fmt::Display::fmt(err, f),
            FlakeError::IO(err) => std::fmt::Display::fmt(err, f),
            FlakeError::AlreadyRunning => {
                f.write_str("Container id in use by another instance, consider @NAME argument")
            }
        }
    }
}

impl Termination for FlakeError {
    /// A failed sub command will forward its error code
    ///
    /// All other errors are represented as Failure
    fn report(self) -> std::process::ExitCode {
        match self {
            FlakeError::CommandError(CommandError {
                base: ProcessError::ExecutionError(Output { status, ..}),
                ..
            }) => match status.code() {
                Some(code) => (code as u8).into(),
                None => ExitCode::FAILURE,
            },
            _ => ExitCode::FAILURE,
        }
    }
}

impl Error for FlakeError {}

impl From<std::io::Error> for FlakeError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

impl From<CommandError> for FlakeError {
    fn from(value: CommandError) -> Self {
        Self::CommandError(value)
    }
}

#[derive(Debug)]
pub enum ProcessError {
    /// The Command failed to execute properly
    IO(std::io::Error),
    // The Command terminated correctly but with unwanted results (e.g. wrong return code)
    ExecutionError(std::process::Output),
}

impl Error for ProcessError {}

impl Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::IO(io) => std::fmt::Display::fmt(&io, f),
            ProcessError::ExecutionError(output) => {
                f.write_str("Failed with code ")?;
                std::fmt::Display::fmt(&output.status, f)
            }
        }
    }
}

impl From<std::process::Output> for ProcessError {
    fn from(value: std::process::Output) -> Self {
        Self::ExecutionError(value)
    }
}

impl From<std::io::Error> for ProcessError {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

#[derive(Debug)]
pub struct CommandError {
    pub base: ProcessError,
    pub args: Vec<String>,
}

impl CommandError {
    pub fn new(base: ProcessError) -> Self {
        Self {
            args: Vec::new(),
            base,
        }
    }

    pub fn with(&mut self, arg: String) -> &mut Self {
        self.args.push(arg);
        self
    }
}

impl Error for CommandError {}

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
        Self {
            base: value.into(),
            args: Default::default(),
        }
    }
}

impl From<ProcessError> for CommandError {
    fn from(value: ProcessError) -> Self {
        Self {
            base: value,
            args: Default::default(),
        }
    }
}

pub trait CommandExtTrait {
    /// Execute this command and return:
    /// 
    /// 1. An IO Error if the command could not be run
    /// 2. An Execution Error if the Command was not successfull
    /// 3. The [Output] of the Command if the command was executed successfully
    /// 
    /// Attaches all args to the resulting error
    fn perform(&mut self) -> Result<std::process::Output, CommandError>;
}

impl CommandExtTrait for Command {
    fn perform(&mut self) -> Result<std::process::Output, CommandError> {
        let out = self.output().map_err(ProcessError::IO);

        let error: ProcessError = match out {
            Ok(output) => {
                if output.status.success() {
                    return Ok(output);
                } else {
                    output.into()
                }
            }
            Err(err) => err.into(),
        };

        Err(CommandError {
            base: error,
            args: self
                .get_args()
                .flat_map(OsStr::to_str)
                .map(ToOwned::to_owned)
                .collect(),
        })
    }
}
