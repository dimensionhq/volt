use crate::classes::package::{Package, Version};
use crate::model::http_manager;
use colored::Colorize;
use sha1;
use std::process;

use crate::__VERSION__;

#[path = "../utils.rs"]
mod utils;

use utils::download_tarbal;

use super::Command;

pub struct Add;

impl Command for Add {
    fn help(&self) -> String {
        format!(
            r#"volt {}
    
Add a package to your dependencies for your project.

Usage: {} {} {} {}

Options: 
    
  {} {} Output the version number.
  {} {} Output verbose messages on internal operations.
  {} {} Disable progress bar."#,
            __VERSION__.bright_green().bold(),
            "volt".bright_green().bold(),
            "add".bright_purple(),
            "[packages]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    fn exec(&self, packages: &Vec<String>, flags: &Vec<String>) {
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

            // println!("package: {:?}", package);

            let version: Version = package
                .versions
                .get_key_value(&package.dist_tags.latest)
                .unwrap()
                .1
                .clone();

            // TODO: Handle Dependencies
            println!("{:?}", version.dependencies);
            // TODO: Download File
            download_tarbal(package);

            // TODO: Verify Checksum
            let dl = sha1::Sha1::from("").digest(); // TODO: Change this to a real checksum

            if dl.to_string() == version.dist.shasum {
                // Verified Checksum
            }
        }
    }
}
