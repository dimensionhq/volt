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

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;

use crate::{utils::App, VERSION};

use super::Command;

/// Struct implementation for the `Remove` command.
pub struct Remove;

#[async_trait]
impl Command for Remove {
    /// Display a help menu for the `volt remove` command.
    fn help(&self) -> String {
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
    async fn exec(&self, _app: Arc<App>, args: Vec<String>, flags: Vec<String>) -> Result<()> {
        println!("Removing packages");
        println!("Packages: {:?}", args);
        println!("Flags: {:?}", flags);
        Ok(())
    }
}
