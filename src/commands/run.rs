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

use crate::{core::VERSION, App, Command as AppCommand};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::process::{Command, Stdio};
use std::sync::Arc;

/// Struct implementation for the `Run` command.
pub struct Run;

#[async_trait]
impl AppCommand for Run {
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
        let script = app.args.value_of("script-name").unwrap().trim();

        if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", "babel"]).spawn().unwrap();
        } else if cfg!(target_os = "linux") {
            println!("{}", format!("$ {}", script).truecolor(156, 156, 156));

            let mut child = Command::new(format!("node_modules/.bin/{}", script))
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("failed to execute child");

            child.wait().unwrap();
        }
        Ok(())
    }
}
