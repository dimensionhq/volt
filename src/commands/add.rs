use crate::model::http_manager;
use crate::traits::UnwrapGraceful;
use crate::utils::{download_tarball, extract_tarball};
use crate::{classes::package::Version, utils::App};
use async_trait::async_trait;
use colored::Colorize;
use sha1::{Digest, Sha1};
use std::io;
use std::{fs::File, sync::Arc};
use tokio::{self, task::JoinHandle};

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

    async fn exec(&self, app: Arc<App>, packages: Vec<String>, flags: Vec<String>) {
        for package_name in packages {
            let package = http_manager::get_package(&package_name)
                .await
                .unwrap_graceful(|err| {
                    format!(
                        "{}: An Error Occured While Requesting {}.json - {}",
                        "error".bright_red().bold(),
                        package_name,
                        format!("{:?}", err).bright_yellow()
                    )
                });

            let version: Version = package
                .versions
                .get_key_value(&package.dist_tags.latest)
                .unwrap()
                .1
                .clone();

            let mut handles: Vec<JoinHandle<()>> = Vec::with_capacity(version.dependencies.len());
            for dependency in version.dependencies.iter() {
                let app = app.clone();
                let dependency = dependency.0.clone();
                let flags = flags.clone();
                let handle = tokio::spawn(async move {
                    println!("Getting dep: {}", &dependency);
                    Add.exec(app, vec![dependency.clone()], flags).await;
                    println!("Done dep: {}", dependency);
                });
                handles.push(handle);
            }

            let app = app.clone();
            handles.push(tokio::spawn(async move {
                let path = download_tarball(&app, &package).await;

                extract_tarball(&path, &package).await.unwrap_graceful(|err| err);

                let mut file = File::open(path).unwrap();
                let mut hasher = Sha1::new();
                io::copy(&mut file, &mut hasher).unwrap();
                let hash = format!("{:x}", hasher.finalize());

                if hash == version.dist.shasum {
                    // Verified Checksum
                    println!("{}", "Successfully Verified Hash".bright_green());
                } else {
                    println!("Failed To Verify")
                }
            }));

            futures::future::join_all(handles).await;
        }
    }
}
