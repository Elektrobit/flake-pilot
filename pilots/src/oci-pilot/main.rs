mod datasync;
mod pdm_tests;
mod podman;

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
