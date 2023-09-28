mod pdm_tests;
mod podman;

fn main() -> Result<(), std::io::Error> {
    match podman::PodmanPilot::new() {
        Ok(pilot) => pilot.start(),
        Err(err) => Err(err),
    }
}
