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
    core::utils::{install_package, State},
    core::{command::Command, VERSION},
    App,
};

use async_trait::async_trait;
use colored::Colorize;
use futures::{stream::FuturesUnordered, StreamExt, TryStreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use miette::Result;
use package_spec::PackageSpec;
use reqwest::Client;
use tokio::task::JoinHandle;

use std::{collections::HashMap, sync::Arc, time::Instant};

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
        let idk = Instant::now();
        // Get input packages
        let mut packages: Vec<PackageSpec> = app.get_packages()?;

        // Load the existing package.json file
        // let (mut package_file, _package_file_path) = PackageJson::open("package.json")?;

        // Construct a path to the local and global lockfile.
        // let lockfile_path = &app.lock_file_path;

        // let global_lockfile = &app.home_dir.join(".global.lock");

        // // Load local and global lockfiles.
        // let mut lock_file =
        //     LockFile::load(lockfile_path).unwrap_or_else(|_| LockFile::new(lockfile_path));

        // let mut global_lock_file =
        //     LockFile::load(global_lockfile).unwrap_or_else(|_| LockFile::new(global_lockfile));

        // let resolve_start = Instant::now();

        let bar = ProgressBar::new_spinner()
            .with_style(ProgressStyle::default_spinner().template("{spinner:.cyan} {msg}"));

        bar.enable_steady_tick(10);

        let resolve_start = Instant::now();

        // Fetch pre-flattened dependency trees from the registry
        let responses = fetch_dep_tree(&packages, &bar).await?;

        let mut tree: HashMap<String, VoltPackage> = HashMap::new();

        for response in responses {
            tree.extend(response.tree);
        }

        let total = tree.len();

        bar.finish_and_clear();

        println!(
            "{} Resolved {} dependencies",
            format!("[{:.2}{}]", resolve_start.elapsed().as_secs_f32(), "s")
                .truecolor(156, 156, 156)
                .bold(),
            total.to_string().truecolor(196, 206, 255).bold()
        );

        let install_start = Instant::now();

        let bar = ProgressBar::new(total as u64);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("[{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
                .progress_chars("=>-"),
        );

        let client = Client::builder().use_rustls_tls().build().unwrap();

        tree.iter()
            .map(|(spec, data)| {
                install_package(
                    app.clone(),
                    &data,
                    State {
                        http_client: client.clone(),
                    },
                )
            })
            .collect::<FuturesUnordered<_>>()
            .inspect(|_| bar.inc(1))
            .try_collect::<Vec<_>>()
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

        // for package in packages {
        //     package_file.add_dependency(package.to_owned());
        // }

        // Save package.json
        // package_file.save()?;

        // Save lockfiles
        // global_lock_file.save()?;
        // lock_file.save()?;
        println!("{}", idk.elapsed().as_secs_f32());
        std::process::exit(0);
        Ok(())
    }
}
