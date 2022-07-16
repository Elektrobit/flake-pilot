#[cfg(test)]
pub mod tests;

pub mod app_path;
pub mod container_link;
pub mod podman;

fn main() {
    let program_path = app_path::program_abs_path();
    let program_name = app_path::basename(&program_path);
    let container_name = container_link::container_name(&program_path);

    podman::run(&program_name, &container_name);
}
