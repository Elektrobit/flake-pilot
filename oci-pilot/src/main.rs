#[cfg(test)]
pub mod tests;

pub mod app_path;
pub mod container_link;
pub mod podman;

fn main() {
    let program_path = app_path::program_abs_path();

    let app_link = container_link::read_link_name(&program_path);

    podman::run(
        &container_link::container_program_path(&app_link, &program_path),
        &container_link::container_name(&app_link)
    );
}
