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
use std::sync::atomic::AtomicI16;
// use std::sync::atomic::Ordering;
use std::sync::Arc;

// Library Imports
use anyhow::{Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use futures::{stream::FuturesUnordered, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::{
    self,
    sync::{mpsc, Mutex},
};

// Crate Level Imports
use crate::utils;
use crate::utils::download_tarball;
use crate::utils::App;
use crate::VERSION;
use crate::{
    classes::package::{Package, Version},
    utils::PROGRESS_CHARS,
};
use crate::{classes::voltapi::VoltPackage, model::lock_file::LockFile};

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

        let lock_file = LockFile::load(app.lock_file_path.to_path_buf())
            .unwrap_or_else(|_| LockFile::new(app.lock_file_path.to_path_buf()));

        // TODO: Change this to handle multiple packages
        let progress_bar: ProgressBar = ProgressBar::new(1);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .progress_chars(PROGRESS_CHARS)
                .template(&format!(
                    "{} [{{bar:40.magenta/blue}}] {{msg:.blue}}",
                    "Fetching dependencies".bright_blue()
                )),
        );

        let response = utils::get_volt_response(app.args[0].to_string()).await;

        let progress_bar = &progress_bar;

        progress_bar.finish_with_message("[OK]".bright_green().to_string());

        let length = &response.versions.len();

        if length.to_owned() == 1 {
            println!("Loaded 1 dependency");
        } else {
            println!("Loaded {} dependencies.", length);
        }

        let mut dependencies: Vec<VoltPackage> = vec![];

        let current_version = response.versions.get(&response.version).unwrap();

        for (_, object) in &current_version.packages {
            dependencies.push(object.clone());
        }

        let mut workers = FuturesUnordered::new();

        for dep in dependencies {
            let app = app.clone();
            workers.push(async move { Add::install_extract_package(app, &dep).await });
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

        // Change package.json
        // for value in &dependencies.to_owned().iter() {
        //     package_file.add_dependency(value.0.name, value.1.version);
        // }

        // Write to lock file
        lock_file.save().context("Failed to save lock file")?;

        Ok(())
    }
}

impl Add {
    // Add new package
    async fn install_extract_package(app: Arc<App>, package: &VoltPackage) -> Result<()> {
        let pb = ProgressBar::new(0);
        let text = format!("{}", "Installing Packages".bright_cyan());

        pb.set_style(
            ProgressStyle::default_spinner()
                .template(("{spinner:.green}".to_string() + format!(" {}", text).as_str()).as_str())
                .tick_strings(&["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"]),
        );

        let tarball_path = download_tarball(&app, &package).await?;

        app.extract_tarball(&tarball_path, &package)
            .await
            .with_context(|| {
                format!("Unable to extract tarball for package '{}'", &package.name)
            })?;

        Ok(())
    }
}
