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
use std::{io, sync::Arc};

// Library Imports
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use sha1::{Digest, Sha1};
use tokio::{self, sync::Mutex};

// Crate Level Imports
use crate::classes::package::{Package, Version};
use crate::model::http_manager;
use crate::model::lock_file::{DependencyLock, LockFile};
use crate::utils::App;
use crate::utils::{download_tarball, extract_tarball};
use crate::VERSION;
use std::sync::atomic::{AtomicBool, Ordering};

// Super Imports
use super::Command;

/// Struct implementation for the `Add` command.
#[derive(Clone)]
pub struct Add {
    lock_file: LockFile,
    dependencies: Arc<Mutex<Vec<(Package, Version)>>>,
}

#[async_trait]
impl Command for Add {
    /// Display a help menu for the `volt add` command.
    fn help() -> String {
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
    ///
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
    async fn exec(app: Arc<App>, packages: Vec<String>, _flags: Vec<String>) -> Result<()> {
        let mut lock_file = LockFile::load(app.lock_file_path.to_path_buf())
            .unwrap_or_else(|_| LockFile::new(app.lock_file_path.to_path_buf()));

        let mut add = Add {
            lock_file: lock_file.clone(),
            dependencies: Arc::new(Mutex::new(Vec::with_capacity(1))),
        };

        for package_name in &packages {
            add.get_dependency_tree(package_name.clone(), None).await?;
        }

        println!("{}", "Generating packages".bright_blue());

        for package_name in packages {
            let (package, version) = Self::fetch_package(&package_name, None).await?;

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
                    dependencies: version.dependencies.clone(),
                },
            );

            let mut handles: Vec<tokio::task::JoinHandle<std::result::Result<(), anyhow::Error>>> =
                Vec::with_capacity(version.dependencies.len());
            add.dependencies
                .lock()
                .map(|deps| {
                    deps.iter()
                        .map(|(dep, ver)| {
                            let app = app.clone();
                            let dependency = dep.name.clone();
                            let handle = tokio::spawn(async move {
                                // println!("Getting dep: {}", &dependency);
                                Add::add_package(app, &dependency).await;
                                // println!("Done dep: {}", &dependency);
                                Result::<_>::Ok(())
                            });
                            handles.push(handle);
                        })
                        .collect::<Vec<_>>()
                })
                .await;

            let progress_bar = ProgressBar::new(9999999);
            let text = format!("{}", "Installing Packages".bright_cyan());

            progress_bar.clone().set_style(
                ProgressStyle::default_spinner()
                    .template(
                        ("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str(),
                    )
                    .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
            );

            let pb = progress_bar.clone();

            let completed = Arc::new(AtomicBool::new(false));

            let completed_clone = completed.clone();

            let handle = tokio::spawn(async move {
                while !completed_clone.load(Ordering::Relaxed) {
                    progress_bar.inc(5);
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                progress_bar.finish_and_clear();
            });

            let app = app.clone();
            handles.push(tokio::spawn(async move {
                let path = download_tarball(&app, &package).await;

                extract_tarball(&path, &package, pb.clone())
                    .await
                    .with_context(|| {
                        format!("Unable to extract tarball for package '{}'", &package.name)
                    })?;

                let mut file = File::open(path).unwrap();
                let mut hasher = Sha1::new();
                io::copy(&mut file, &mut hasher).unwrap();
                let hash = format!("{:x}", hasher.finalize());

                if hash == version.dist.shasum {
                    // Verified Checksum
                    // pb.println(format!("{}", "Successfully Verified Hash".bright_green()));
                } else {
                    pb.println(format!("{}", "Failed To Verify".bright_red()));
                }

                Result::<_>::Ok(())
            }));

            for handle in handles {
                let _ = handle.await;
            }
            completed.store(true, Ordering::Relaxed);
            let _ = handle.await;
        }

        // Write to lock file
        lock_file.save().context("Failed to save lock file")?;

        Ok(())
    }
}

impl Add {
    async fn add_package(app: Arc<App>, package_name: &str) {
        let pb = ProgressBar::new(9999999);
        let text = format!("{}", "Installing Packages".bright_cyan());

        pb.set_style(
            ProgressStyle::default_spinner()
                .template(("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str())
                .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
        );

        let (package, version) = Self::fetch_package(package_name, None).await.unwrap();

        let tarball_path = download_tarball(&app, &package).await;
        let _ = extract_tarball(&tarball_path, &package, pb.clone())
            .await
            .with_context(|| format!("Unable to extract tarball for package '{}'", &package.name));
        let mut file = File::open(tarball_path).unwrap();
        let mut hasher = Sha1::new();
        io::copy(&mut file, &mut hasher).unwrap();
        let hash = format!("{:x}", hasher.finalize());
        if hash == version.dist.shasum {
            // Verified Checksum
            // pb.println(format!("{}", "Successfully Verified Hash".bright_green()));
        } else {
            pb.println(format!(
                "{} {}",
                "Failed To Verify Checksum For".bright_red(),
                &package.name
            ));
        }
    }
    async fn fetch_package(
        package_name: &str,
        version_req: Option<semver::VersionReq>,
    ) -> Result<(Package, Version)> {
        let package = http_manager::get_package(&package_name)
            .await
            .with_context(|| format!("Failed to fetch package '{}'", package_name))?
            .ok_or_else(|| {
                anyhow!(
                    "Package '{}' was not found or is not available",
                    package_name
                )
            })?;

        let version: Version = match &version_req {
            Some(req) => {
                let mut available_versions: Vec<semver::Version> = package
                    .versions
                    .iter()
                    .filter_map(|(k, _)| k.parse().ok())
                    .collect();
                available_versions.sort();
                available_versions.reverse();

                available_versions
                    .into_iter()
                    .find(|v| req.matches(v))
                    .map(|v| package.versions.get(&v.to_string()))
                    .flatten()
            }
            None => package.versions.get(&package.dist_tags.latest),
        }
        .ok_or_else(|| {
            if let Some(req) = version_req {
                anyhow!(
                    "Version {} for '{}' is not found",
                    req.to_string(),
                    &package_name
                )
            } else {
                anyhow!("Unable to find latest version for '{}'", &package_name)
            }
        })?
        .clone();

        Ok((package, version))
    }

    fn get_dependency_tree(
        &mut self,
        package_name: String,
        version_req: Option<semver::VersionReq>,
    ) -> BoxFuture<'_, Result<()>> {
        async move {
            let pkg = Self::fetch_package(&package_name, version_req).await?;
            let pkg_deps = pkg.1.dependencies.clone();

            self.dependencies
                .lock()
                .map(|mut deps| {
                    if !deps.iter().any(|(package, version)| {
                        package.name == pkg.0.name && pkg.1.version == version.version
                    }) {
                        deps.push(pkg);
                    }
                })
                .await;

            let mut workers = FuturesUnordered::new();

            for (name, version) in pkg_deps {
                let pkg_name = name.clone();
                let mut self_copy = self.clone();
                workers.push(tokio::spawn(async move {
                    self_copy
                        .get_dependency_tree(
                            pkg_name,
                            Some(
                                version
                                    .parse()
                                    .map_err(|_| anyhow!("Could not parse dependency version"))?,
                            ),
                        )
                        .await
                }));
            }

            loop {
                match workers.next().await {
                    Some(result) => result??,
                    None => break,
                }
            }

            Ok(())
        }
        .boxed()
    }
}
