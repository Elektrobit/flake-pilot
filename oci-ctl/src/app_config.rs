use std::fs;
use std::path::Path;
extern crate yaml_rust;

use yaml_rust::YamlLoader;

// constants related to field names in configuration 
const CONTAINER:&str = "container";
const TARGET_APP_PATH:&str = "target_app_path";
const HOST_APP_PATH:&str = "host_app_path";

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

/*
AppConfig represents application yaml configuration
*/
pub struct AppConfig {
    pub container: String,
    pub target_app_path: String,
    pub host_app_path: String,
    pub config_file: String
}

impl AppConfig {
    pub fn save(
        config_file: &Path, container: &String, target_app_path: &String,
        host_app_path: &String
    ) -> Result<(), GenericError> {
        /*!
        save stores an AppConfig to the given file
        !*/
        // TODO: handling yaml string can be done better here...
        let app_config = format!(
            "container: {}\ntarget_app_path: {}\nhost_app_path: {}\n",
            &container,
            &target_app_path,
            &host_app_path
        );
        fs::write(&config_file, app_config)?;
        Ok(())
    }

    pub fn init_from_file(config_file: &Path) -> Result<AppConfig, GenericError> {
        /*!
        new creates the new AppConfig class by reading and
        deserializing the data from a given yaml configuration
        !*/
        let mut rs = AppConfig{
            container: "".to_string(),
            target_app_path: "".to_string(),
            host_app_path: "".to_string(),
            config_file: config_file.display().to_string()
        };

        if Path::new(&config_file).exists() {
            let source = fs::read_to_string(&config_file)?;
            let doc = YamlLoader::load_from_str(&source)?;

            rs.container = match doc[0][CONTAINER].as_str() {
                Some(v) => v.to_string(),
                None => "".to_string()
            };

            rs.target_app_path = match doc[0][TARGET_APP_PATH].as_str() {
                Some(v) => v.to_string(),
                None => "".to_string()
            };

            rs.host_app_path = match doc[0][HOST_APP_PATH].as_str() {
                Some(v) => v.to_string(),
                None => "".to_string()
            };
        }
        Ok(rs)
    }
}
