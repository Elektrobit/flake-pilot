use std::env;

use flakes::logger;

mod datasync;
mod datatracker;
mod pdm_tests;
mod podman;
mod prunner;

static VERSION: &str = "0.1";
static LOGGER: logger::STDOUTLogger = logger::STDOUTLogger;

fn main() -> Result<(), std::io::Error> {
    // Setup logger
    if let Err(err) = log::set_logger(&LOGGER).map(|()| {
        log::set_max_level(if !env::var("DEBUG").unwrap_or("".to_string()).is_empty() {
            log::LevelFilter::Trace
        } else {
            log::LevelFilter::Info
        })
    }) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()));
    }

    log::debug!("Launching pilot");
    match podman::PodmanPilot::new() {
        Ok(pilot) => match pilot.start() {
            Ok(_) => Ok(()),
            Err(err) => {
                println!("General error: {}", err);
                Err(err)
            }
        },
        Err(err) => Err(err),
    }
}
