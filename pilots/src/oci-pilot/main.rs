mod datasync;
mod datatracker;
mod pdm_tests;
mod podman;
mod prunner;

fn main() -> Result<(), std::io::Error> {
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
