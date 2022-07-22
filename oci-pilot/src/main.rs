#[cfg(test)]
pub mod tests;

pub mod app_path;
pub mod container_link;
pub mod podman;

fn main() {
    let program_path = app_path::program_abs_path();

    podman::run(
        &container_link::read_link_container_program_path(&program_path),
        &container_link::read_link_container_name(&program_path)
    );
}
