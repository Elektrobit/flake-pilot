use std::path::Path;
use std::fs;

pub fn read_link(program_path: &String) -> Vec<String> {
    /*!
    Read container name and app path from program_path symlink
    registered by oci-register and return a vector
    of the form:

    [container_name, container_app_path]

    Supported link names are:

    - /usr/bin/app -> /usr/share/flakes/container@|usr|bin|app
    - /usr/bin/app -> /usr/share/flakes/container
    !*/
    let program_link_path = fs::read_link(program_path)
        .expect(&format!("Must be a container symlink: {}", program_path));

    let link_names: Vec<&str> = Path::new(
        &program_link_path
    ).file_name().unwrap().to_str().unwrap().split("@").collect();

    // extract container name from link name split
    let mut result: Vec<String> = Vec::new();
    result.push(
        String::from(link_names[0])
    );

    // extract container app from link name split or use program_path
    if link_names.len() > 1 {
        result.push(
            String::from(link_names[1]).chars().map(|c| match c {
                '|' => '/',
                _ => c
            }).collect()
        );
    } else {
        result.push(program_path.to_string());
    }
    result
}
