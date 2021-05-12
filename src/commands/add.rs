use crate::{classes::package::Version, utils::download_tarbal};
use crate::{model::http_manager, traits::UnwrapGraceful};
use async_trait::async_trait;
use colored::Colorize;
use sha1;

use crate::__VERSION__;

use super::Command;

pub struct Add;

#[async_trait]
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

    async fn exec(&self, packages: &Vec<String>, _flags: &Vec<String>) {
        for package_name in packages {
            let package = http_manager::get_package(package_name)
                .await
                .unwrap_graceful(|err| {
                    format!(
                        "{}: An Error Occured While Requesting {}.json - {}",
                        "error".bright_red().bold(),
                        package_name,
                        err.to_string().bright_yellow()
                    )
                });

            // println!("package: {:?}", package);

            let version: Version = package
                .versions
                .get_key_value(&package.dist_tags.latest)
                .unwrap()
                .1
                .clone();

            // TODO: Handle Dependencies

            // TODO: Download File
            download_tarbal(&package).await;

            // TODO: Verify Checksum
            let dl = sha1::Sha1::from("").digest(); // TODO: Change this to a real checksum

            if dl.to_string() == version.dist.shasum {
                // Verified Checksum
            }
        }
    }
}
