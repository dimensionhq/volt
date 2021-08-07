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

use std::io::Write;
use std::process::exit;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use tokio::sync::Mutex;
use utils::error;
use utils::{app::App, npm::get_versions};

use utils::package::PackageJson;

use volt_core::{command::Command, VERSION};

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
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```rust
    /// // Add react to your dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app, vec!["react"], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        // Display help menu if `volt add` is run.
        if app.args.len() == 1 {
            println!("{}", Self::help());
            exit(0);
        }

        let mut packages = vec![];

        // Add packages to the packages vec.
        for arg in app.args.iter() {
            if arg != "add" {
                packages.push(arg.clone());
            }
        }

        // Check if package.json exists, otherwise, handle it.
        if !app.current_dir.join("package.json").exists() {
            error!("no package.json found.");
            print!("Do you want to initialize package.json (Y/N): ");

            std::io::stdout().flush().expect("Could not flush stdout");

            let mut input: String = String::new();

            std::io::stdin().read_line(&mut input).unwrap();

            if input.trim().to_lowercase() != "y" || input.trim().to_lowercase() != "yes" {
                exit(0);
            } else {
                init::command::Init::exec(app.clone()).await.unwrap();
            }
        }

        // Load the existing package.json file
        let package_file = Arc::new(Mutex::new(PackageJson::from("package.json")));

        let versions = get_versions(&packages).await?;

        println!("{:?}", versions);

        Ok(())
    }
}
