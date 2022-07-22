use std::path::Path;
use std::fs;

pub fn read_link_name(program_path: &String) -> String {
    /*!
    Read container link, supported link names are:

    /usr/bin/app -> /usr/share/flakes/container@|usr|bin|app
    /usr/bin/app -> /usr/share/flakes/container
    !*/
    let program_link_name = fs::read_link(program_path)
        .expect(&format!("Must be a container symlink: {}", program_path));
    let mut result = String::new();
    result.push_str(
        Path::new(&program_link_name).file_name().unwrap().to_str().unwrap()
    );
    result
}

pub fn read_link_container_program_path(program_path: &String) -> String {
    /*!
    Read container program path from program_path symlink
    registered by oci-register
    !*/
    let link_name = read_link_name(&program_path);
    let link_names: Vec<&str> = Path::new(
        &link_name
    ).file_name().unwrap().to_str().unwrap().split("@").collect();

    // extract container app from link name split or use program_path
    let container_program_path: String;
    if link_names.len() > 1 {
        container_program_path = String::from(
            link_names[1]).chars().map(|c| match c {
                '|' => '/',
                _ => c
            }
        ).collect();
    } else {
        container_program_path = program_path.to_string();
    }
    let mut result = String::new();
    result.push_str(&container_program_path);
    result
}

pub fn read_link_container_name(program_path: &String) -> String {
    /*!
    Read container name from program_path symlink
    registered by oci-register
    !*/
    let link_name = read_link_name(&program_path);
    let link_names: Vec<&str> = Path::new(
        &link_name
    ).file_name().unwrap().to_str().unwrap().split("@").collect();

    // extract container name from link name split
    let mut result = String::new();
    result.push_str(link_names[0]);
    result
}
