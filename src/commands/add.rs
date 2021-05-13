use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use sha1::{Digest, Sha1};
use std::io;
use std::{fs::File, sync::Arc};
use tokio::{self, task::JoinHandle};

use crate::model::lock::LockFile;
use crate::model::{http_manager, lock::DependencyLock};
use crate::utils::{download_tarball, extract_tarball};
use crate::__VERSION__;
use crate::{classes::package::Version, utils::App};

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

    async fn exec(&self, app: Arc<App>, packages: Vec<String>, _flags: Vec<String>) -> Result<()> {
        let mut lock_file = LockFile::load(app.lock_file_path.to_path_buf())
            .unwrap_or_else(|_| LockFile::new(app.lock_file_path.to_path_buf()));

        for package_name in packages {
            let package = http_manager::get_package(&package_name)
                .await
                .with_context(|| format!("Failed to fetch package '{}'", package_name))?
                .ok_or_else(|| {
                    anyhow!(
                        "Package '{}' was not found or is not available",
                        package_name
                    )
                })?;

            let version: Version = package
                .versions
                .get_key_value(&package.dist_tags.latest)
                .unwrap()
                .1
                .clone();

            lock_file.dependencies.push(DependencyLock {
                name: package_name.clone(),
                version: package.clone().dist_tags.latest,
                tarball: version.clone().dist.tarball,
                sha1: version.clone().dist.shasum,
            });

            let mut handles: Vec<JoinHandle<Result<()>>> =
                Vec::with_capacity(version.dependencies.len());

            // for dependency in version.dependencies.iter() {
            //     let app = app.clone();
            //     let dependency = dependency.0.clone();
            //     let flags = flags.clone();
            //     let handle = tokio::spawn(async move {
            //         println!("Getting dep: {}", &dependency);
            //         Add.exec(app, vec![dependency.clone()], flags).await;
            //         println!("Done dep: {}", dependency);
            //     });
            //     handles.push(handle);
            // }

            let app = app.clone();
            handles.push(tokio::spawn(async move {
                let path = download_tarball(&app, &package).await;

                extract_tarball(&path, &package).await.with_context(|| {
                    format!("Unable to extract tarbal for package '{}'", &package.name)
                })?;

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

                Result::<_>::Ok(())
            }));

            futures::future::join_all(handles).await;
        }

        // Write to lock file
        lock_file.save().context("Failed to save lock file")?;

        Ok(())
    }
}
