use std::{process::{Command, ExitCode, Stdio}, convert::Infallible, env::Args, io::stdout, path::Path};

fn main() -> Result<ExitCode, std::io::Error>{
    let mut args = std::env::args();
    let name = args.next().map(format!());
    match name {
        Some(name) => {
            if Command::new(name).args(args).status()?.success() {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::FAILURE)
            }

        },
        None => help(),
    }
}

fn help() -> Result<ExitCode, std::io::Error>{
    Ok(ExitCode::SUCCESS)
}
