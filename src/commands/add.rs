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
use std::{process::exit, sync::atomic::AtomicI16};
// use std::sync::atomic::Ordering;
use std::sync::Arc;

// Library Imports
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde_json::Value;
use tokio::{
    self,
    sync::{mpsc, Mutex},
};

// Crate Level Imports
use crate::model::lock_file::LockFile;
use crate::utils::download_tarball;
use crate::utils::App;
use crate::VERSION;
use crate::{
    classes::package::{Package, Version},
    utils::PROGRESS_CHARS,
};
use crate::{model::http_manager, utils::get_basename};

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
        // let package_file = PackageJson::from("package.json");
        if app.args.len() == 0 {
            println!("{}", Self::help());
            exit(1);
        }
        let mut verbose = app.has_flag(&["-v", "--verbose"]);
        let pballowed = app.has_flag(&["--no-progress", "-np"]);

        let lock_file = LockFile::load(app.lock_file_path.to_path_buf())
            .unwrap_or_else(|_| LockFile::new(app.lock_file_path.to_path_buf()));

        let (tx, _) = mpsc::channel(100);
        let add = Add::new(lock_file.clone(), tx);
        if pballowed {
            println!(
                "{} {} {}",
                "Setting verbose to".blue(),
                "true".green(),
                "as progressbar is disabled".blue()
            );
        }
        let packages = app.args.clone();
        if !pballowed {
            let progress_bar: ProgressBar = ProgressBar::new(packages.len() as u64);
            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .progress_chars(PROGRESS_CHARS)
                    .template(&format!(
                        "{} [{{bar:40.magenta/blue}}] {{msg:.blue}}",
                        "Fetching dependencies".bright_blue()
                    )),
            );
            let progress_bar = &progress_bar;
            // This will improve add times too
            // for package_name in packages {
            //     let mut add = add.clone();
            //     workers.push(async move {
            //         add.get_dependencies(package_name.clone(), None)
            //             .await
            //             .map(|_| progress_bar.inc(1))
            //     });
            // }

            // loop {
            //     match workers.next().await {
            //         Some(result) => result?,
            //         None => break,
            //     }
            // }

            progress_bar.finish_with_message("[OK]".bright_green().to_string());
        } else {
            // progressbar disabled hence verbose= true
            verbose = true
        }

        if verbose {
            println!("info: {}", "Fetching Dependency tree".yellow())
        }
        // let mut workers = FuturesUnordered::new();

        let length = add
            .clone()
            .dependencies
            .lock()
            .map(|deps| {
                deps.iter()
                    .map(|(dep, ver)| format!("{}: {}", dep.name, ver.version))
                    .collect::<Vec<_>>()
                    .len()
            })
            .await;

        if length == 1 {
            println!("Loaded 1 dependency");
        } else {
            println!("Loaded {} dependencies.", length);
        }
        if verbose {
            println!(
                "info {} {}",
                "Got dependency tree adding them to project".yellow(),
                String::from(get_basename(app.current_dir.to_owned().to_str().unwrap())).yellow()
            );
        }
        let dependencies = Arc::try_unwrap(add.clone().dependencies)
            .map_err(|_| anyhow!("Unable to read dependencies"))?
            .into_inner();

        let mut workers = FuturesUnordered::new();

        for (dep, ver) in dependencies {
            let app = app.clone();
            workers.push(async move { Add::install_extract_package(app, &dep, &ver).await });
        }
        if !pballowed {
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
        } else {
            loop {
                match workers.next().await {
                    Some(result) => {
                        result?;
                    }

                    None => break,
                }
            }
        }

        // Change package.json
        // for value in &dependencies.to_owned().iter() {
        //     package_file.add_dependency(value.0.name, value.1.version);
        // }

        // Write to lock file
        if verbose {
            println!("info {}", "Writing to lock file".yellow());
        }
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

        let tarball_path = download_tarball(&app, &package, &version).await?;

        app.extract_tarball(&tarball_path, &package)
            .await
            .with_context(|| {
                format!("Unable to extract tarball for package '{}'", &package.name)
            })?;

        Ok(())
    }

    async fn fetch_package(
        package_name: &str,
        version_req: Option<semver::VersionReq>,
    ) -> Result<(Package, Version)> {
        let package_name = package_name.replace("\"", "");

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

    // fn get_dependency_tree(
    //     &mut self,
    // package_name: String,
    // version_req: Option<semver::VersionReq>,
    // ) -> BoxFuture<'_, Result<()>> {
    //     async move {
    //         let pkg = Self::fetch_package(&package_name, version_req).await?;
    //         let pkg_deps = pkg.1.dependencies.clone();

    //         let should_download = self
    //             .dependencies
    //             .lock()
    //             .map(|mut deps| {
    //                 if !deps.iter().any(|(package, version)| {
    //                     package.name == pkg.0.name && pkg.1.version == version.version
    //                 }) {
    //                     deps.push(pkg);
    //                     true
    //                 } else {
    //                     false
    //                 }
    //             })
    //             .await;

    //         if !should_download {
    //             return Ok(());
    //         }

    //         let mut workers = FuturesUnordered::new();

    //         self.total_dependencies.store(
    //             self.total_dependencies.load(Ordering::Relaxed) + 1,
    //             Ordering::Relaxed,
    //         );

    //         for (name, version) in pkg_deps {
    //             // Increase total
    //             self.total_dependencies.store(
    //                 self.total_dependencies.load(Ordering::Relaxed) + 1,
    //                 Ordering::Relaxed,
    //             );

    //             let pkg_name = name.clone();
    //             let mut self_copy = self.clone();
    //             workers.push(tokio::spawn(async move {
    //                 let res = self_copy
    //                     .get_dependency_tree(
    //                         pkg_name,
    //                         Some(
    //                             version
    //                                 .parse()
    //                                 .map_err(|_| anyhow!("Could not parse dependency version"))?,
    //                         ),
    //                     )
    //                     .await;
    //                 // Increase completed
    //                 self_copy.progress_sender.send(()).await.ok();

    //                 res
    //             }));
    //         }

    //         loop {
    //             match workers.next().await {
    //                 Some(result) => result??,
    //                 None => break,
    //             }
    //         }

    //         self.progress_sender.send(()).await.ok();

    //         Ok(())
    //     }
    //     .boxed()
    // }

    // async fn get_dependencies(
    //     &mut self,
    //     package_name: String,
    //     version_req: Option<semver::VersionReq>,
    // ) -> Result<()> {
    //     let (package, version) = Add::fetch_package(package_name.as_str(), None).await?;

    //     let dependencies = &package
    //         .versions
    //         .get(package.dist_tags.latest.as_str())
    //         .unwrap()
    //         .dependencies;

    //     if dependencies.len() > 0 {
    //         let response = http_manager::get_dependencies(package_name.as_str()).await;
    //         let data: Value = serde_json::from_str(response.as_str()).unwrap();

    //         let mut dependencies = vec![];

    //         if version_req.is_some() {
    //             let deps: Vec<String> = data["dependencies"][version_req.unwrap().to_string()]
    //                 .as_array()
    //                 .ok_or_else(|| {
    //                     anyhow::Error::msg("Failed to parse dependencies from server response.")
    //                 })?
    //                 .into_iter()
    //                 .map(|value| value.to_string())
    //                 .collect();

    //             let mut workers = FuturesUnordered::new();

    //             for dep in deps.iter() {
    //                 workers.push(async move {
    //                     let data = Add::fetch_package(dep.as_str(), None)
    //                         .await
    //                         .context("Failed to fetch a package from the registry.")?;

    //                     Result::<(Package, Version), anyhow::Error>::Ok(data)
    //                 });
    //             }

    //             loop {
    //                 match workers.next().await {
    //                     Some(result) => dependencies.push(result?),
    //                     None => break,
    //                 }
    //             }

    //             Ok(())
    //         } else {
    //             // Get latest version
    //             let latest_version = &data["dependencies"]
    //                 .as_object()
    //                 .ok_or_else(|| {
    //                     anyhow::Error::msg(
    //                         "Failed to parse dependencies from server response. [latest_version]",
    //                     )
    //                 })?
    //                 .keys()
    //                 .into_iter()
    //                 .map(|value| value.to_string())
    //                 .collect::<Vec<String>>()[0];

    //             let deps: Vec<String> = data["dependencies"][latest_version.to_owned()]
    //                 .as_object()
    //                 .ok_or_else(|| {
    //                     anyhow::Error::msg(
    //                         "Failed to parse dependencies from server response. [deps]",
    //                     )
    //                 })?
    //                 .keys()
    //                 .into_iter()
    //                 .map(|value| value.to_string())
    //                 .collect();

    //             let mut workers = FuturesUnordered::new();

    //             for dep in deps.iter() {
    //                 workers.push(async move {
    //                     // println!("Getting: {}", dep);
    //                     let data = Add::fetch_package(dep.as_str(), None)
    //                         .await
    //                         .context("Failed to fetch a package from the registry.")?;
    //                     // println!("Got: {}", dep);
    //                     Result::<(Package, Version), anyhow::Error>::Ok(data)
    //                 });
    //             }

    //             for dep in deps.iter() {
    //                 workers.push(async move {
    //                     let data = Add::fetch_package(dep.as_str(), None)
    //                         .await
    //                         .context("Failed to fetch a package from the registry.")?;
    //                     Result::<(Package, Version), anyhow::Error>::Ok(data)
    //                 });
    //             }

    //             loop {
    //                 match workers.next().await {
    //                     Some(result) => dependencies.push(result?),
    //                     None => break,
    //                 }
    //             }

    //             dependencies.push((package, version));

    //             self.dependencies = Arc::new(Mutex::new(dependencies));

    //             Ok(())
    //         }
    //     } else {
    //         self.dependencies = Arc::new(Mutex::new(vec![(package, version)]));
    //         Ok(())
    //     }
    // }
}
