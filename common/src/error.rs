use std::process::{ExitCode, Output, Termination};
use thiserror::Error;

use crate::command::{CommandError, ProcessError};

#[derive(Debug, Error)]
pub enum FlakeError {
    /// The pilot tried to run a sub command and failed
    #[error("Failed to run {}", .0)]
    CommandError(#[from] CommandError),
    /// There was an error in an IO operation
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[cfg(feature = "json")]
    #[error(transparent)]
    MalformedJson(#[from] serde_json::Error),
    /// This flake is already running
    #[error("Instance in use by another instance, consider @NAME argument")]
    AlreadyRunning,
    #[error("{}", .0)]
    OperationError(#[from] OperationError)
}

#[derive(Debug, Error)]
pub enum OperationError {
    #[error("Max retries for VM connection check exceeded")]
    MaxTriesExceeded
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

