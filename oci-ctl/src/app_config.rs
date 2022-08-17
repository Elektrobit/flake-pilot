use std::fs;
use std::path::Path;
extern crate yaml_rust;

use yaml_rust::{YamlLoader, Yaml};

/* constants related to field names in configuration */
const CONTAINER_NAME:&str = "container_name";
const PROGRAM_NAME:&str   = "program_name";

type GenericError = Box<dyn std::error::Error + Send + Sync + 'static>;

    /*
     * AppConfig represents application yaml configuration
     * and storing container name and program name 
     */
pub struct AppConfig{
    pub container_name: String,
    pub program_name: String,
}

impl AppConfig {

    pub fn new( conf_name: &Path ) -> Result< AppConfig, GenericError> {
        /*!
         * new creates the new AppConfig class by reading and deserializing the data
         * from a given yaml configuration 
         !*/
        let mut rs = AppConfig{container_name: "".to_string() ,program_name: "".to_string()};
        
        let source = fs::read_to_string(&conf_name)?;
    
        let doc = YamlLoader::load_from_str(&source)?;

        rs.container_name = match doc[0][CONTAINER_NAME].as_str(){
            Some(v) => v.to_string(),
            None => "".to_string()
        };
        
        rs.program_name = match doc[0][PROGRAM_NAME].as_str() {
            Some(v) => v.to_string(),
            None => "".to_string()
        };
        
        Ok( rs )        
    }

    pub fn save( &mut self, conf_name: &Path) -> Result< (), GenericError >{
        /*!
         * save stores the AppConfig object to the given file
         */       
        let content = match Yaml::Array(vec!(Yaml::from_str(&self.container_name),Yaml::from_str(&self.program_name))).into_string() {
            Some(v) => v,
            None => return Err(GenericError::from("Wrong content stored in yaml configuration")),
        };
        fs::write(conf_name, content)?;
        Ok(())
    }
}
