use flakes::logger;
use std::env;

mod fgc;
mod pdm_tests;
mod pdsys;
mod podman;
mod prunner;

static LOGGER: logger::STDOUTLogger = logger::STDOUTLogger;

fn main() -> Result<(), std::io::Error> {
    // Setup logger
    let debug = !env::var("DEBUG").unwrap_or("".to_string()).is_empty();
    if let Err(err) = log::set_logger(&LOGGER)
        .map(|()| log::set_max_level(if debug { log::LevelFilter::Trace } else { log::LevelFilter::Info }))
    {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()));
    }

    log::debug!("Launching pilot");

    match podman::PodmanPilot::new(debug) {
        Ok(mut pilot) => match pilot.start() {
            Ok(_) => {}
            Err(err) => {
                log::error!("General error: {}", err);
            }
        },
        Err(err) => {
            log::error!("Unable to start flake: {}", err);
        }
    }

    Ok(())
}
