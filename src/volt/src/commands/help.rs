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

use std::sync::Arc;

use async_trait::async_trait;
use colored::Colorize;
use miette::DiagnosticResult;
use utils::app::App;
use volt_core::{command::Command, VERSION};
/// Struct implementation for the `Help` command.
pub struct Help;

#[async_trait]
impl Command for Help {
    /// Display a help menu for the `volt help` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Displays help information.

Usage: {} {} {}

Commands:

  {} {} - Install all dependencies for a project.
  {} {} - Interactively create or update a package.json file for a project.
  {} {} - Add a dependency to a project.
  {} {} - Lists the dependency tree of a project.
  {} {} - Remove a dependency from the package.json file for a project.
  {} {} - Push changes to a github repository the easy way.
  {} {} - Clean the volt cache files and metadata.
  {} {} - Clone a github repository and get setup with all required dependencies.
  {} {} - Run a defined script.
  "#,
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
            "list".bright_blue(),
            "*".bright_magenta().bold(),
            "remove".bright_blue(),
            "*".bright_magenta().bold(),
            "cache".bright_blue(),
            "*".bright_magenta().bold(),
            "deploy".bright_blue(),
            "*".bright_magenta().bold(),
            "clone".bright_blue(),
            "*".bright_magenta().bold(),
            "run".bright_blue(),
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
    async fn exec(_app: Arc<App>) -> DiagnosticResult<()> {
        println!("{}", Self::help());
        Ok(())
    }
}
