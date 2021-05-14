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

//! Displays help information.

// Std Imports
use std::sync::Arc;

// Library Imports
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;

// Crate Level Imports
use crate::utils::App;
use crate::VERSION;

// Super Imports
use super::Command;

/// Struct implementation for the `Help` command.
pub struct Help;

#[async_trait]
impl Command for Help {
    /// Display a help menu for the `volt help` command.
    fn help(&self) -> String {
        format!(
            r#"volt {}
    
Displays help information.

Usage: {} {} {}

Commands:

  {} {} - Install all dependencies for a project.
  {} {} - Interactively create or update a package.json file for a project.
  {} {} - Add a dependency to a project.
  {} {} - Remove a dependency from the package.json file for a project."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "[commands]".bright_purple(),
            "[flags]".bright_purple(),
            "*".bright_magenta().bold(),
            "install".bright_blue(),
            "*".bright_magenta().bold(),
            "init".bright_blue(),
            "*".bright_magenta().bold(),
            "add".bright_blue(),
            "*".bright_magenta().bold(),
            "remove".bright_blue()
        )
    }

    /// Execute the `volt help` command
    ///
    /// Displays help information.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Display a help menu.
    /// // .exec() is an async call so you need to await it
    /// Help.exec(app, vec![], vec![]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(&self, _app: Arc<App>, _args: Vec<String>, _flags: Vec<String>) -> Result<()> {
        println!("{}", self.help());
        Ok(())
    }
}
