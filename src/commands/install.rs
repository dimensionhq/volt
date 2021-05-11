use crate::classes::package::{Package, Version};
use crate::model::http_manager;
use colored::Colorize;
use std::process;

use crate::__VERSION__;

use super::Command;

pub struct Install;

impl Command for Install {
    fn help(&self) -> String {
        format!(
            r#"volt {}
        
    Install dependencies for a project.
    
    Usage: {} {} {}
        
    Options: 
        
      {} {} Accept all prompts while installing dependencies.  
      {} {} Output verbose messages on internal operations."#,
            __VERSION__.bright_green().bold(),
            "volt".bright_green().bold(),
            "install".bright_purple(),
            "[flags]".white(),
            "--yes".blue(),
            "(-y)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    fn exec(&self, packages: &Vec<String>, _flags: &Vec<String>) {
        for package_name in packages {
            let response = match http_manager::get_package(package_name) {
                Ok(text) => text,
                Err(e) => {
                    eprintln!(
                        "{}: An Error Occured While Requesting {}.json - {}",
                        "error".bright_red().bold(),
                        package_name,
                        e.to_string().bright_yellow()
                    );
                    process::exit(1);
                }
            };
            let package: Package = serde_json::from_str(&response).unwrap();
            let version: Version = package
                .versions
                .get_key_value(&package.dist_tags.latest)
                .unwrap()
                .1
                .clone();
        }
    }
}
