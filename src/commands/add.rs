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
use std::{fs::File, sync::atomic::AtomicI16};
use std::{io, sync::Arc};

// Library Imports
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use futures::{future::BoxFuture, stream::FuturesUnordered, FutureExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use sha1::{Digest, Sha1};
use tokio::{
    self,
    sync::{mpsc, Mutex},
    task::JoinHandle,
};

// Crate Level Imports
use crate::classes::package::{Package, Version};
use crate::model::http_manager;
use crate::model::lock_file::LockFile;
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
    total_dependencies: Arc<AtomicI16>,
    progress_sender: mpsc::Sender<()>,
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
        let lock_file = LockFile::load(app.lock_file_path.to_path_buf())
            .unwrap_or_else(|_| LockFile::new(app.lock_file_path.to_path_buf()));

        let (tx, mut rx) = mpsc::channel(100);
        let add = Add::new(lock_file.clone(), tx);

        {
            let mut add = add.clone();
            let packages = packages.clone();
            tokio::spawn(async move {
                for package_name in packages {
                    add.get_dependency_tree(package_name.clone(), None)
                        .await
                        .ok();
                }
            });
        }

        let progress_bar = ProgressBar::new(1);

        progress_bar.set_style(ProgressStyle::default_bar().progress_chars("=> ").template(
            &format!(
                "{} [{{bar:40.cyan/blue}}] {{len}} {{msg:.green}}",
                "Fetching dependencies".bright_cyan()
            ),
        ));

        let mut done: i16 = 0;
        while let Some(_) = rx.recv().await {
            done += 1;
            let total = add.total_dependencies.load(Ordering::Relaxed);
            if done == total {
                break;
            }
            progress_bar.set_length(total as u64);
            progress_bar.set_position(done as u64);
        }
        progress_bar.finish_with_message("[DONE]");

        println!(
            "Loaded {} dependencies.",
            add.dependencies
                .lock()
                .map(|deps| deps
                    .iter()
                    .map(|(dep, ver)| format!("{}: {}", dep.name, ver.version))
                    .collect::<Vec<_>>()
                    .len())
                .await
        );

        let dependencies = Arc::try_unwrap(add.dependencies)
            .map_err(|_| anyhow!("Unable to read dependencies"))?
            .into_inner();

        let mut handles: Vec<JoinHandle<std::result::Result<(), anyhow::Error>>> =
            Vec::with_capacity(dependencies.len());

        for (dep, ver) in dependencies {
            let app = app.clone();
            // let d_clone = dep.clone();
            let version = ver.clone();
            let dependency = dep.name.clone();
            let handle = tokio::spawn(async move {
                // println!("Getting dep: {}", &dependency);
                Add::add_package(app, &dependency).await;
                // println!("Completed: {}", &dependency);
                Result::<_>::Ok(())
            });
            handles.push(handle);
        }

        let progress_bar = ProgressBar::new(9999999);
        let text = format!("{}", "Installing Packages".bright_cyan());

        progress_bar.clone().set_style(
            ProgressStyle::default_spinner()
                .template(("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str())
                .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
        );

        let completed = Arc::new(AtomicBool::new(false));

        let completed_clone = completed.clone();

        let handle = tokio::spawn(async move {
            while !completed_clone.load(Ordering::Relaxed) {
                progress_bar.inc(5);
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
            progress_bar.finish_and_clear();
        });

        // let app = app.clone();
        // handles.push(tokio::spawn(async move {
        //     let path = download_tarball(&app, &package).await;

        //     extract_tarball(&path, &package, pb.clone())
        //         .await
        //         .with_context(|| {
        //             format!("Unable to extract tarball for package '{}'", &package.name)
        //         })?;

        //     let mut file = File::open(path).unwrap();
        //     let mut hasher = Sha1::new();
        //     io::copy(&mut file, &mut hasher).unwrap();
        //     let hash = format!("{:x}", hasher.finalize());

        //     if hash == version.dist.shasum {
        //         // Verified Checksum
        //         // pb.println(format!("{}", "Successfully Verified Hash".bright_green()));
        //     } else {
        //         pb.println(format!("{}", "Failed To Verify".bright_red()));
        //     }

        //     Result::<_>::Ok(())
        // }));

        for handle in handles {
            let _ = handle.await;
        }
        completed.store(true, Ordering::Relaxed);
        let _ = handle.await;

        // Write to lock file
        lock_file.save().context("Failed to save lock file")?;

        Ok(())
    }
}

impl Add {
    fn new(lock_file: LockFile, progress_sender: mpsc::Sender<()>) -> Self {
        Self {
            lock_file,
            dependencies: Arc::new(Mutex::new(Vec::with_capacity(1))),
            total_dependencies: Arc::new(AtomicI16::new(0)),
            progress_sender,
        }
    }

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
                &package.name.bright_red()
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

            let should_download = self
                .dependencies
                .lock()
                .map(|mut deps| {
                    if !deps.iter().any(|(package, version)| {
                        package.name == pkg.0.name && pkg.1.version == version.version
                    }) {
                        deps.push(pkg);
                        true
                    } else {
                        false
                    }
                })
                .await;

            if !should_download {
                return Ok(());
            }

            let mut workers = FuturesUnordered::new();

            self.total_dependencies.store(
                self.total_dependencies.load(Ordering::Relaxed) + 1,
                Ordering::Relaxed,
            );

            for (name, version) in pkg_deps {
                // Increase total
                self.total_dependencies.store(
                    self.total_dependencies.load(Ordering::Relaxed) + 1,
                    Ordering::Relaxed,
                );

                let pkg_name = name.clone();
                let mut self_copy = self.clone();
                workers.push(tokio::spawn(async move {
                    let res = self_copy
                        .get_dependency_tree(
                            pkg_name,
                            Some(
                                version
                                    .parse()
                                    .map_err(|_| anyhow!("Could not parse dependency version"))?,
                            ),
                        )
                        .await;

                    // Increase completed
                    self_copy.progress_sender.send(()).await.ok();

                    res
                }));
            }

            loop {
                match workers.next().await {
                    Some(result) => result??,
                    None => break,
                }
            }

            self.progress_sender.send(()).await.ok();

            Ok(())
        }
        .boxed()
    }
}
