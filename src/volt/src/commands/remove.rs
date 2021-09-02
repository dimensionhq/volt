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

use std::{
    io::Write,
    process,
    sync::Arc,
};

use crate::commands::init::Init;

use async_trait::async_trait;
use colored::Colorize;
use miette::DiagnosticResult;
use tokio::sync::Mutex;
use utils::{
    app::App,
    error,
    package::PackageJson,
};
use volt_core::{
    command::Command,
    model::lock_file::LockFile,
    VERSION,
};
/// Struct implementation for the `Remove` command.
pub struct Remove;

#[async_trait]
impl Command for Remove
{
    /// Display a help menu for the `volt remove` command.
    fn help() -> String
    {
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
    async fn exec(app: Arc<App>) -> DiagnosticResult<()>
    {
        if app.args.len() == 1 {
            println!("{}", Self::help());
            process::exit(1);
        }

        let mut packages = vec![];
        for arg in &app.args {
            if arg != "remove" {
                packages.push(arg.clone());
            }
        }

        let package_json_dir = std::env::current_dir().unwrap().join("package.json");

        if !package_json_dir.exists() {
            error!("no package.json found");
            print!("Do you want to initialize package.json (Y/N): ");
            std::io::stdout().flush().expect("Could not flush stdout");
            let mut string: String = String::new();
            let _ = std::io::stdin().read_line(&mut string);
            if string.trim().to_lowercase() != "y" {
                process::exit(0);
            } else {
                Init::exec(app.clone()).await.unwrap();
            }
        }

        let package_file = Arc::new(Mutex::new(PackageJson::from("package.json")));

        // let mut handles = vec![];

        println!("{}", "Removing dependencies".bright_purple());

        for package in packages {
            let package_file = package_file.clone();
            let app_new = app.clone();

            // handles.push(tokio::spawn(async move {
            let mut package_json_file = package_file.lock().await;

            package_json_file.dependencies.remove(&package);

            package_json_file.save();

            let mut lock_file = LockFile::load(app_new.lock_file_path.to_path_buf())
                .unwrap_or_else(|_| LockFile::new(app_new.lock_file_path.to_path_buf()));

            // let response = get_volt_response(&package).await?;

            // let current_version = response.versions.get(&response.version).unwrap();

            // for object in current_version.values() {
            //     // This is doing nothing? I guess it's still WIP?...
            //     // let mut lock_dependencies: HashMap<String, String> = HashMap::new();

            //     // if object.dependencies.is_some() {
            //     //     for dep in object.clone().dependencies.unwrap().iter() {
            //     //         // TODO: Change this to real version
            //     //         lock_dependencies.insert(dep.clone(), String::new());
            //     //     }
            //     // }

            //     lock_file
            //         .dependencies
            //         .remove(&DependencyID(object.clone().name, object.clone().version));

            //     let scripts = Path::new("node_modules/scripts")
            //         .join(format!("{}.cmd", object.clone().name).as_str());

            //     if scripts.exists() {
            //         remove_file(format!(
            //             "node_modules/scripts/{}",
            //             scripts.file_name().unwrap().to_str().unwrap()
            //         ))
            //         .await
            //         .unwrap_and_handle_error();
            //     }
            // }

            // lock_file.save().unwrap();

            // let node_modules_dir = std::env::current_dir().unwrap().join("node_modules");
            // let dep_dir = node_modules_dir.join(&package);
            // if dep_dir.exists() {
            //     remove_dir_all(dep_dir).await.unwrap_and_handle_error();
            // }
        }

        // if handles.len() > 0 {
        //     for handle in handles {
        //         handle.await?;
        //     }
        // }

        println!("{}", "Successfully Removed Packages".bright_blue());

        Ok(())
    }
}
