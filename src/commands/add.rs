use crate::classes::package::Version;
use crate::model::http_manager;
use crate::traits::UnwrapGraceful;
use crate::utils::{download_tarball, extract_tarball};
use async_trait::async_trait;
use colored::Colorize;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io;

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

            let version: Version = package
                .versions
                .get_key_value(&package.dist_tags.latest)
                .unwrap()
                .1
                .clone();

            // TODO: Handle Dependencies
            // for dependency in version.dependencies.iter() {
            //     self.exec(vec![dependency], flags);
            // }

            let path = download_tarball(&package).await;

            match extract_tarball(path.as_str(), &package) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("{}", err);
                }
            };

            let mut file = File::open(path).unwrap();
            let mut hasher = Sha1::new();
            io::copy(&mut file, &mut hasher).unwrap();
            let hash = format!("{:x}", hasher.finalize());

            if hash == version.dist.shasum {
                // Verified Checksum
                println!("{}", "Successfully Verified Hash".bright_green());
            }
        }
    }
}
