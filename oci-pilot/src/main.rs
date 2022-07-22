#[cfg(test)]
pub mod tests;

pub mod app_path;
pub mod container_link;
pub mod podman;

fn main() {
    let program_path = app_path::program_abs_path();
    let container_meta = container_link::read_link(&program_path);

    let container_name = &container_meta[0];
    let container_program_path = &container_meta[1];

    podman::run(&container_program_path, &container_name);
}
