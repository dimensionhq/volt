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

//! Remove a package from your direct dependencies.

// Std Imports
use std::sync::Arc;

// Library Imports
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;

// Crate Level Imports
use crate::utils::App;
use crate::VERSION;

use std::process::exit;

use std::collections::HashMap;

use std::io::Write;

use crate::commands::init;

use crate::utils::get_volt_response;

use crate::classes::package::PackageJson;
use crate::model::lock_file::{DependencyID, DependencyLock, LockFile};

use tokio::{self, sync::Mutex};

// Super Imports
use super::Command;

/// Struct implementation for the `Remove` command.
pub struct Remove;

#[async_trait]
impl Command for Remove {
    /// Display a help menu for the `volt remove` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Removes a package from your direct dependencies.

Usage: {} {} {} {}

Options: 

  {} {} Output the version number.
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "remove".bright_purple(),
            "[packages]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt remove` command
    ///
    /// Removes a package from your direct dependencies.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Remove a package from your direct dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Remove.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        if app.args.len() == 1 {
            println!("{}", Self::help());
            exit(1);
        }

        let mut packages = vec![];
        for arg in &app.args {
            if arg != "remove" {
                packages.push(arg.clone());
            }
        }

        let package_json_dir = std::env::current_dir()?.join("package.json");

        if !package_json_dir.exists() {
            println!("{} no package.json found", "error".bright_red());
            print!("Do you want to initialize package.json (Y/N): ");
            std::io::stdout()
                .flush()
                .ok()
                .expect("Could not flush stdout");
            let mut string: String = String::new();
            let _ = std::io::stdin().read_line(&mut string);
            if string.trim().to_lowercase() != "y" {
                exit(0);
            } else {
                init::Init::exec(app.clone()).await.unwrap();
            }
        }

        let package_file = Arc::new(Mutex::new(PackageJson::from("package.json")));

        let mut handles = vec![];

        for package in packages {
            let package_file = package_file.clone();
            let app_new = app.clone();

            handles.push(tokio::spawn(async move {
                let mut package_json_file = package_file.lock().await;

                package_json_file
                .dependencies.remove(&package);

                package_json_file.save();

                let mut lock_file = LockFile::load(app_new.lock_file_path.to_path_buf())
                .unwrap_or_else(|_| LockFile::new(app_new.lock_file_path.to_path_buf()));

                let response = get_volt_response(package.to_string()).await;

                let current_version = response.versions.get(&response.version).unwrap();

                for (_, object) in &current_version.packages {
                    let mut lock_dependencies: HashMap<String, String> = HashMap::new();

                    if object.clone().dependencies.is_some() {
                        for dep in object.clone().dependencies.unwrap().iter() {
                            // TODO: Change this to real version
                            lock_dependencies.insert(dep.clone(), String::new());
                        }
                    }

                    let hashmap: HashMap<DependencyID, DependencyLock> = [(DependencyID(object.clone().name, object.clone().version),
                    DependencyLock {
                        name: object.clone().name,
                        version: object.clone().version,
                        tarball: object.clone().tarball,
                        sha1: object.clone().sha1,
                        dependencies: lock_dependencies,
                    })].iter().cloned().collect();

                    lock_file.dependencies.remove(
                        &DependencyID(object.clone().name, object.clone().version)
                    );
                }                

                lock_file
                .save()                
                .unwrap();

                println!("lock: {:?}", lock_file);
            }));
        }

        if handles.len() > 0 {
            for handle in handles {
                handle.await?;
            }
        }

        Ok(())
    }
}
