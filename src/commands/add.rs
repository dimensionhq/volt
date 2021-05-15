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
use std::sync::atomic::Ordering;
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
};

// Crate Level Imports
use crate::model::http_manager;
use crate::model::lock_file::LockFile;
use crate::utils::download_tarball;
use crate::utils::App;
use crate::VERSION;
use crate::{
    classes::package::{Package, Version},
    utils::PROGRESS_CHARS,
};

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
    async fn exec(app: Arc<App>) -> Result<()> {
        let lock_file = LockFile::load(app.lock_file_path.to_path_buf())
            .unwrap_or_else(|_| LockFile::new(app.lock_file_path.to_path_buf()));

        let (tx, mut rx) = mpsc::channel(100);
        let add = Add::new(lock_file.clone(), tx);

        {
            let mut add = add.clone();
            let packages = app.args.clone();
            tokio::spawn(async move {
                for package_name in packages {
                    add.get_dependency_tree(package_name.clone(), None)
                        .await
                        .ok();
                }
            });
        }

        let progress_bar = ProgressBar::new(1);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .progress_chars(PROGRESS_CHARS)
                .template(&format!(
                    "{} [{{bar:40.magenta/blue}}] {{msg:.blue}}",
                    "Fetching dependencies".bright_blue()
                )),
        );

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
        progress_bar.finish_with_message("[OK]".bright_green().to_string());

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

        let mut workers = FuturesUnordered::new();

        for (dep, ver) in dependencies {
            let app = app.clone();
            workers.push(async move { Add::install_extract_package(app, &dep, &ver).await });
        }

        let progress_bar = ProgressBar::new(workers.len() as u64);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .progress_chars(PROGRESS_CHARS)
                .template(&format!(
                    "{} [{{bar:40.magenta/blue}}] {{msg:.blue}} {{pos}} / {{len}}",
                    "Installing packages".bright_blue()
                )),
        );

        loop {
            match workers.next().await {
                Some(result) => {
                    result?;
                    progress_bar.inc(1)
                }

                None => break,
            }
        }

        progress_bar.finish();

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

    // Add new package
    async fn install_extract_package(
        app: Arc<App>,
        package: &Package,
        version: &Version,
    ) -> Result<()> {
        let pb = ProgressBar::new(0);
        let text = format!("{}", "Installing Packages".bright_cyan());

        pb.set_style(
            ProgressStyle::default_spinner()
                .template(("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str())
                .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
        );

        let tarball_path = download_tarball(&app, &package, &version.version).await;

        app.extract_tarball(&tarball_path, &package)
            .await
            .with_context(|| {
                format!("Unable to extract tarball for package '{}'", &package.name)
            })?;

        let mut file = File::open(tarball_path).context("Unable to open tar file")?;
        let mut hasher = Sha1::new();
        io::copy(&mut file, &mut hasher).context("Unable to read tar file")?;
        let hash = format!("{:x}", hasher.finalize());
        if hash == version.dist.shasum {
            // Verified Checksum
            // pb.println(format!("{}", "Successfully Verified Hash".bright_green()));
        } else {
            println!(
                "Got hash: {} but it should've been {}",
                hash, version.dist.shasum
            );
            pb.println(format!(
                "{} {}",
                "Failed to verify checksum for".bright_red(),
                &package.name.bright_red()
            ));
        }

        Ok(())
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
