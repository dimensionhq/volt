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

use std::sync::Arc;

// use crate::commands::scripts::Script;
use crate::core::VERSION;
use crate::App;
use crate::Command;

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

/// Struct implementation for the `Run` command.
pub struct Run;

#[async_trait]
impl Command for Run {
    /// Display a help menu for the `volt run` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Run a pre-defined package script

Usage: {} {} {}
    
Options:
    
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "run".bright_purple(),
            "file-name".white(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt run` command
    ///
    /// Interactively create or update a package.json file for a project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Run a defined script.
    /// // .exec() is an async call so you need to await it
    /// Run.exec(app, vec![], vec!["--yes"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        // if app.clone().args.len() == 1_usize {
        //     let package_json = PackageJson::from("package.json");

        //     let args = app.args.clone();
        //     let command: &str = args[0].as_str();

        //     if package_json.scripts.contains_key(command) {
        //         Script::exec(app.clone()).await.unwrap();
        //         std::process::exit(0);
        //     }
        // }

        // let path = Path::new("node_modules/scripts");

        // if path.exists() {
        //     let files = read_dir("node_modules/scripts").unwrap();

        //     let mut files_vec: Vec<String> = vec![];

        //     for f in files {
        //         let f = f.unwrap();
        //         let file_name = f.file_name();
        //         let file_name_str = file_name.to_string_lossy();

        //         files_vec.push(file_name_str.to_string());
        //     }

        //     if app.clone().args.len() == 1_usize {
        //         let print_string = files_vec.join(", ");
        //         println!(
        //             "{}{} {}",
        //             "scripts".bright_cyan().bold(),
        //             ":".bright_magenta().bold(),
        //             print_string
        //         );
        //         std::process::exit(1);
        //     }

        //     if files_vec.contains(&app.args[1]) {
        //         let location = format!("node_modules/scripts/{}", &app.args[1]);

        //         let command = format!("scripts/{}", &app.args[1]);
        //         println!("{} {}", ">".bright_magenta().bold(), command);

        //         std::process::Command::new("cmd.exe")
        //             .arg("/C")
        //             .arg(location.replace("/", r"\"))
        //             .spawn()
        //             .unwrap();
        //     } else {
        //         error!(
        //             "{} 'is not a valid script.'",
        //             &app.args[1].bright_yellow().bold(),
        //         );
        //     }
        // }

        Ok(())
    }
}
