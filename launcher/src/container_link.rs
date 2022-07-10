use std::path::Path;
use std::env;
use std::fs;

pub fn container_name(program_path: &String) -> String {
    /*!
    Read container name from program_path symlink.
    This function expects the symlink layout for containers
    to be applied.

    There is also the magic option @CONTAINERNAME evaluated
    from the commandline arguments which allows to override
    the container name
    !*/
    let args: Vec<String> = env::args().collect();
    let program_link_path = fs::read_link(program_path)
        .expect(&format!("Must be a container symlink: {}", program_path));
    let mut container_name = String::new();
    container_name.push_str(
        Path::new(&program_link_path).file_name().unwrap().to_str().unwrap()
    );
    // allow to override containername via @CONTAINERNAME
    for arg in &args[1..] {
        if arg.starts_with("@") {
            container_name = String::from(&arg.replace("@", ""));
        }
    }
    container_name
}
