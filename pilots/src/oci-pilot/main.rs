mod pdm_tests;
mod podman;

fn main() {
    match podman::PodmanPilot::new() {
        Ok(pilot) => {}
        Err(err) => {
            panic!("{}", err)
        }
    }
}
