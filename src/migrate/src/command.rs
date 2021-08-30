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

//!  Migrates a package from your direct dependencies.

use std::{env, fs, process, sync::Arc};

use async_trait::async_trait;
use colored::Colorize;
use miette::DiagnosticResult;
use utils::{app::App, error};
use volt_core::{
    classes::package_manager::PackageManager, command::Command, prompt::prompts::Select, VERSION,
};
/// Struct implementation for the `Migrate` command.
pub struct Migrate;

#[async_trait]
impl Command for Migrate {
    /// Display a help menu for the `volt migrate` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Migrates a project to either yarn,volt,npm or pnpm from your current proj.

Usage: {} {} {} {}

Options: 

  {} {} Output the version number.
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "migrate".bright_purple(),
            "[package_manager_name]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt migrate` command
    ///
    /// Migrates a project to either yarn,volt,npm or pnpm from your current proj.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `package_manager_name` - The  (`String`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Migrate a package from your direct dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Migrate.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> DiagnosticResult<()> {
        // let packagemanagers: Vec<String> = PackageManager::options();
        // let mut packagemanager: String = String::new();
        // if app.args.len() == 1 {
        //     packagemanager = app.args[0].to_string();
        // } else if app.args.len() == 1 {
        //     let select = Select {
        //         message: String::from("Package Manager"),
        //         paged: true,
        //         selected: Some(1),
        //         items: packagemanagers.clone(),
        //     };
        //     let selected = select.run().unwrap_or_else(|err| {
        //         error!("{}", err.to_string());
        //         process::exit(1);
        //     });

        //     packagemanager = PackageManager::from_index(selected).unwrap().to_string();
        // } else {
        //     error!("{}", "volt migrate only takes 1 argument");
        // }

        // if packagemanager.eq_ignore_ascii_case("volt") {
        //     std::fs::remove_dir_all("node_modules").unwrap();

        //     let files = fs::read_dir(env::current_dir().unwrap()).unwrap();
        //     files
        //         .filter_map(Result::ok)
        //         .filter(|d| {
        //             if let Some(e) = d.path().extension() {
        //                 String::from(e.to_str().unwrap()).contains("lock")
        //             } else {
        //                 false
        //             }
        //         })
        //         .for_each(|f| std::fs::remove_file(f.file_name()).unwrap());
        //     println!("{}", "$ volt install".truecolor(147, 148, 148));
        //     install::command::Install::exec(app).await?; // NOTE WILL ONLY WORK IF THE VAR DEPENDENCIES is populated
        // } else if packagemanager.eq_ignore_ascii_case("yarn") {
        //     std::fs::remove_dir_all("node_modules").unwrap();

        //     let files = fs::read_dir(env::current_dir().unwrap()).unwrap();
        //     files
        //         .filter_map(Result::ok)
        //         .filter(|d| {
        //             if let Some(e) = d.path().extension() {
        //                 e == "lock"
        //             } else {
        //                 false
        //             }
        //         })
        //         .for_each(|f| std::fs::remove_file(f.file_name()).unwrap());

        //     println!("{}", "$ yarn".truecolor(147, 148, 148));
        //     std::process::Command::new("yarn")
        //         .spawn()
        //         .expect("failed to execute")
        //         .wait()
        //         .unwrap();
        // } else if packagemanager.eq_ignore_ascii_case("pnpm") {
        //     std::fs::remove_dir_all("node_modules").unwrap();

        //     let files = fs::read_dir(env::current_dir().unwrap()).unwrap();
        //     files
        //         .filter_map(Result::ok)
        //         .filter(|d| {
        //             if let Some(e) = d.path().file_name() {
        //                 String::from(e.to_str().unwrap()).contains("lock")
        //             } else {
        //                 false
        //             }
        //         })
        //         .for_each(|f| std::fs::remove_file(f.file_name()).unwrap());

        //     println!("{}", "$ pnpm install".truecolor(147, 148, 148));
        //     std::process::Command::new("pnpm")
        //         .arg("install")
        //         .spawn()
        //         .expect("failed to execute")
        //         .wait()
        //         .unwrap();
        // } else if packagemanager.eq_ignore_ascii_case("npm") {
        //     std::fs::remove_dir_all("node_modules").unwrap();

        //     let files = fs::read_dir(env::current_dir().unwrap()).unwrap();
        //     files
        //         .filter_map(Result::ok)
        //         .filter(|d| {
        //             if let Some(e) = d.path().file_name() {
        //                 String::from(e.to_str().unwrap()).contains("lock")
        //             } else {
        //                 false
        //             }
        //         })
        //         .for_each(|f| std::fs::remove_file(f.file_name()).unwrap());

        //     println!("{}", "$ npm install".truecolor(147, 148, 148));
        //     std::process::Command::new("npm")
        //         .arg("install")
        //         .spawn()
        //         .expect("failed to execute")
        //         .wait()
        //         .unwrap();
        // } else {
        //     println!("Volt accepts only volt, yarn, pnpm or npm for volt migrate's args it does not support {}" ,app.args[0].to_string().red());
        // }
        Ok(())
    }
}
