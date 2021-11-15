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

//! Add a package to the dependencies for your project.

use crate::{
    core::model::lock_file::{DependencyID, DependencyLock, LockFile},
    core::utils::voltapi::VoltPackage,
    core::utils::{fetch_dep_tree, package::PackageJson},
    core::utils::{install_package, npm, State},
    core::{command::Command, VERSION},
    App,
};

use async_trait::async_trait;
use colored::Colorize;
use futures::{stream::FuturesUnordered, StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;

use std::{sync::Arc, time::Instant};

#[derive(Clone, Debug)]
pub struct PackageInfo {
    pub name: String,
    pub version: Option<String>,
}

/// Struct implementation for the `Add` command.
#[derive(Clone)]
pub struct Add {}

#[async_trait]
impl Command for Add {
    /// Display a help menu for the `volt add` command.
    fn help() -> String {
        format!(
            r#"volt {}

            Add a package to your project's dependencies.
            Usage: {} {} {} {}
            Options:

            {} {} Output the version number.
            {} {} Output verbose messages on internal operations.
            {} {} Adds package as a dev dependency
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
            "--dev".blue(),
            "(-D)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    /// Execute the `volt add` command
    ///
    /// Adds a package to dependencies for your project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```rust
    /// // Add react to your dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        // Get input packages
        let mut packages = app.get_packages()?;

        // Load the existing package.json file
        let (mut package_file, _package_file_path) = PackageJson::open("package.json")?;

        // Construct a path to the local and global lockfile.
        let lockfile_path = &app.lock_file_path;

        let global_lockfile = &app.home_dir.join(".global.lock");

        // Load local and global lockfiles.
        let mut lock_file =
            LockFile::load(lockfile_path).unwrap_or_else(|_| LockFile::new(lockfile_path));

        let mut global_lock_file =
            LockFile::load(global_lockfile).unwrap_or_else(|_| LockFile::new(global_lockfile));

        let resolve_start = Instant::now();

        let bar = ProgressBar::new_spinner()
            .with_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}"))
            .with_message(packages[0].name.clone());

        bar.enable_steady_tick(10);

        // Fetch npm data including hash to fetch dependencies
        let data = npm::get_versions(&packages, &bar).await?;

        // Fetch pre-flattened dependency trees from the registry
        let responses = fetch_dep_tree(&data, &bar).await?;

        let mut dependencies: Vec<VoltPackage> = vec![];

        let mut total = 0;

        for res in responses.iter() {
            for package in res.versions.values().into_iter() {
                total += 1;
                dependencies.push(package.to_owned());
            }
        }

        bar.finish_and_clear();

        println!(
            "{} Resolved {} dependencies",
            format!("[{:.2}{}]", resolve_start.elapsed().as_secs_f32(), "s")
                .truecolor(156, 156, 156)
                .bold(),
            total.to_string().truecolor(196, 206, 255).bold()
        );

        let mut dependencies: Vec<_> = dependencies
            .iter()
            .map(|object| {
                let mut lock_dependencies: Vec<String> = vec![];

                if let Some(peer_deps) = &object.peer_dependencies {
                    for dep in peer_deps {
                        if !crate::core::utils::check_peer_dependency(dep) {
                            println!(
                                "{}: {} has unmet peer dependency {}",
                                "warn".bright_yellow(),
                                object.name.bright_cyan(),
                                &dep.bright_yellow()
                            );
                        }
                    }
                }

                if let Some(dependencies) = &object.dependencies {
                    for dep in dependencies {
                        lock_dependencies.push(dep.to_string());
                    }
                }

                let object_instance = object.clone();

                lock_file.dependencies.insert(
                    DependencyID(object_instance.name, object_instance.version),
                    DependencyLock {
                        name: object.name.clone(),
                        version: object.version.clone(),
                        tarball: object.tarball.clone(),
                        integrity: object.integrity.clone(),
                        dependencies: lock_dependencies.clone(),
                    },
                );

                let second_instance = object.clone();

                global_lock_file.dependencies.insert(
                    DependencyID(second_instance.name, second_instance.version),
                    DependencyLock {
                        name: object.name.clone(),
                        version: object.version.clone(),
                        tarball: object.tarball.clone(),
                        integrity: object.integrity.clone(),
                        dependencies: lock_dependencies,
                    },
                );

                object
            })
            .collect();

        for dep in dependencies.iter() {
            for package in packages.iter_mut() {
                if dep.name == package.name {
                    package.version = Some(dep.version.clone());
                }
            }
        }

        // Remove duplicate dependencies
        dependencies.dedup();

        let install_start = Instant::now();

        let bar = ProgressBar::new(dependencies.len() as u64);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .progress_chars("=>-"),
        );

        dependencies
            .into_iter()
            .map(|v| install_package(&app, v, State {}))
            .collect::<FuturesUnordered<_>>()
            .inspect(|_| bar.inc(1))
            .try_collect::<()>()
            .await
            .unwrap();

        bar.finish_and_clear();

        println!(
            "{} Installed {} dependencies",
            format!("[{:.2}{}]", install_start.elapsed().as_secs_f32(), "s")
                .truecolor(156, 156, 156)
                .bold(),
            total.to_string().truecolor(196, 206, 255).bold()
        );

        for package in packages {
            package_file.add_dependency(package);
        }

        // Save package.json
        package_file.save()?;

        // Save lockfiles
        global_lock_file.save()?;
        lock_file.save()?;

        Ok(())
    }
}
