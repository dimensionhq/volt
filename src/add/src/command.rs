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

use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use std::{process::exit, sync::atomic::AtomicI16};

use anyhow::{Context, Result};
use async_trait::async_trait;
use colored::Colorize;
use futures::stream;
use futures::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::{mpsc, Mutex};
use utils::app::App;
use utils::volt_api::{VoltPackage, VoltResponse};
use utils::{
    self,
    package::{Package, PackageJson, Version},
    PROGRESS_CHARS,
};
use volt_core::{
    command::Command,
    model::lock_file::{DependencyID, DependencyLock, LockFile},
    VERSION,
};

/// Struct implementation for the `Add` command.
#[derive(Clone)]
pub struct Add {
    lock_file: LockFile,
    dependencies: Arc<Mutex<Vec<(Package, Version)>>>,
    dev_dependencies: Arc<Mutex<Vec<(Package, Version)>>>,
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
        // // Display help menu if `volt add` is run.
        if &app.args.len() == &1 {
            println!("{}", Self::help());
            exit(1);
        }

        let mut packages = vec![];

        // Add packages to the packages vec.
        for arg in &app.args {
            if arg != "add" {
                packages.push(arg.clone());
            }
        }

        // Check if package.json exists, otherwise, handle it.
        if !&app.current_dir.join("package.json").exists() {
            println!("{} no package.json found.", "error".bright_red());
            print!("Do you want to initialize package.json (Y/N): ");
            std::io::stdout().flush().expect("Could not flush stdout");
            let mut string: String = String::new();
            let _ = std::io::stdin().read_line(&mut string);
            if string.trim().to_lowercase() != "y" {
                exit(0);
            } else {
                init::command::Init::exec(app.clone()).await.unwrap();
            }
        }

        // Load the existing package.json file
        let package_file = Arc::new(Mutex::new(PackageJson::from("package.json")));

        // Iterate through each package
        let app_instance = app.clone();
        let package_file = package_file.clone();

        let verbose = app_instance.has_flag(&["-v", "--verbose"]);

        let pballowed = !app_instance.has_flag(&["--no-progress", "-np"]);

        let lcp = app_instance.lock_file_path.to_path_buf();

        let global_lockfile = Path::new(&format!(
            r"{}/volt-global.lock",
            app_instance.home_dir.display()
        ))
        .to_path_buf();

        let mut lock_file = LockFile::load(lcp.clone()).unwrap_or_else(|_| LockFile::new(lcp));

        let mut global_lockfile = LockFile::load(global_lockfile.clone())
            .unwrap_or_else(|_| LockFile::new(global_lockfile));

        // TODO: Change this to handle multiple packages
        let progress_bar: ProgressBar = ProgressBar::new(1);

        progress_bar.set_style(
            ProgressStyle::default_bar()
                .progress_chars(PROGRESS_CHARS)
                .template(&format!(
                    "{} [{{bar:40.magenta/blue}}] {{msg:.blue}}",
                    "Resolving dependencies".bright_blue()
                )),
        );

        let responses: Vec<VoltResponse>;

        let start = Instant::now();

        if packages.len() > 1 {
            responses = utils::get_volt_response_multi(packages.clone()).await;
        } else {
            responses = vec![utils::get_volt_response(packages[0].to_string()).await];
        }

        let end = Instant::now();

        progress_bar.finish_with_message("[OK]".bright_green().to_string());
        let progress_bar = &progress_bar;

        let mut dependencies: HashMap<String, VoltPackage> = HashMap::new();

        for res in responses.iter() {
            let current_version = res.versions.get(&res.version).unwrap();

            dependencies.extend(current_version.clone().packages);
        }

        let length = dependencies.len();

        if length == 1 {
            println!(
                "{}: resolved 1 dependency in {:.2}s.\n",
                "success".bright_green(),
                (end - start).as_secs_f32()
            );
        } else {
            println!(
                "{}: resolved {} dependencies in {:.2}s.\n",
                "success".bright_green(),
                length,
                (end - start).as_secs_f32()
            );
        }

        let dependencies: Vec<_> = dependencies
            .iter()
            .map(|(_name, object)| {
                let mut lock_dependencies: Vec<String> = vec![];
                let object_instance = object.clone();

                object_instance
                    .peer_dependencies
                    .into_iter()
                    .for_each(|dep| {
                        if !utils::check_peer_dependency(&dep) {
                            progress_bar.println(format!(
                                "{}{} {} has unmet peer dependency {}",
                                " warn ".black().on_bright_yellow(),
                                ":",
                                object.name.bright_cyan(),
                                &dep.bright_yellow()
                            ));
                        }
                    });

                if object.dependencies.is_some() {
                    for dep in object_instance.dependencies.unwrap().iter() {
                        // TODO: Change this to real version
                        lock_dependencies.push(dep.to_string());
                    }
                }

                lock_file.dependencies.insert(
                    DependencyID(object.clone().name, object.clone().version.to_owned()),
                    DependencyLock {
                        name: object_instance.name,
                        version: object_instance.version,
                        tarball: object_instance.tarball,
                        sha1: object_instance.sha1,
                        dependencies: lock_dependencies.clone(),
                    },
                );

                let second_instance = object.clone();

                global_lockfile.dependencies.insert(
                    DependencyID(object.clone().name, object.clone().version.to_owned()),
                    DependencyLock {
                        name: second_instance.name,
                        version: second_instance.version,
                        tarball: second_instance.tarball,
                        sha1: second_instance.sha1,
                        dependencies: lock_dependencies,
                    },
                );

                object
            })
            .collect();

        progress_bar.finish_and_clear();

        let mut workers = Vec::with_capacity(dependencies.len());

        for dep in dependencies.iter() {
            let app_instance = app_instance.clone();
            workers.push(async move {
                utils::install_extract_package(&app_instance, &dep)
                    .await
                    .unwrap();
            });
        }

        let stream = stream::iter(workers);
        let mut buffers = stream.buffer_unordered(23);

        if pballowed {
            let progress_bar = ProgressBar::new(dependencies.len() as u64);

            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .progress_chars(PROGRESS_CHARS)
                    .template(&format!(
                        "{} [{{bar:40.magenta/blue}}] {{msg:.blue}} {{pos}} / {{len}}",
                        "Installing Packages".bright_blue()
                    )),
            );

            while buffers.next().await.is_some() {
                progress_bar.inc(1);
            }

            progress_bar.finish();
        } else {
            while buffers.next().await.is_some() {}
        }

        let mut package_json_file = package_file.lock().await;

        if app_instance.flags.contains(&"-D".to_string())
            || app_instance.flags.contains(&"--dev".to_string())
        {
            for (idx, package) in packages.clone().into_iter().enumerate() {
                package_json_file
                    .dev_dependencies
                    .insert(package.to_string(), format!("^{}", &responses[idx].version));
            }
        } else {
            for (idx, package) in packages.clone().into_iter().enumerate() {
                package_json_file
                    .dev_dependencies
                    .insert(package.to_string(), format!("^{}", &responses[idx].version));
            }
        }

        package_json_file.save();

        // Write to lock file
        if verbose {
            println!("info {}", "Writing to lock file".yellow());
        }

        lock_file
            .save()
            .context("Failed to save lock file")
            .unwrap();

        global_lockfile
            .save()
            .context("Failed to save global lockfile")
            .unwrap();

        Ok(())
    }
}
