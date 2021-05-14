/*
Copyright 2021 Volt Contributors

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

//! Add a package to your dependencies for your project.

// Std Imports
use std::fs::File;
use std::io;
use std::sync::Arc;

// Library Imports
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use sha1::{Digest, Sha1};
use tokio::{self, task::JoinHandle};

// Crate Level Imports
use crate::classes::package::Version;
use crate::model::http_manager;
use crate::model::lock_file::{DependencyLock, LockFile};
use crate::utils::App;
use crate::utils::{download_tarball, extract_tarball};
use crate::VERSION;

// Super Imports
use super::Command;

/// Struct implementation for the `Add` command.
pub struct Add;

#[async_trait]
impl Command for Add {
    /// Display a help menu for the `volt add` command.
    fn help(&self) -> String {
        format!(
            r#"volt {}
    
Add a package to your dependencies for your project.

Usage: {} {} {} {}

Options: 
    
  {} {} Output the version number.
  {} {} Output verbose messages on internal operations.
  {} {} Disable progress bar."#,
            VERSION.bright_green().bold(),
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

    /// Execute the `volt add` command
    /// Adds a package to dependencies for your project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Add react to your dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app, vec!["react"], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
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

            lock_file.add(
                (
                    package_name.clone(),
                    format!("^{}", package.dist_tags.latest),
                ),
                DependencyLock {
                    name: package_name.clone(),
                    version: package.dist_tags.latest.clone(),
                    tarball: version.dist.tarball.clone(),
                    sha1: version.dist.shasum.clone(),
                },
            );

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
