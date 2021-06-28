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

//! Check for errors

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use volt_core::{command::Command, VERSION};
use volt_utils::app::App;
/// Struct implementation for the `Deploy` command.
pub struct Fix;

#[async_trait]
impl Command for Fix {
    /// Display a help menu for the `volt deploy` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Checks for errors.

Usage: {} {} {}

Options: 

  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "deploy".bright_purple(),
            "[commit]".white(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt fix` command
    ///
    /// Removes a package from your direct dependencies.
    /// ## Examples
    /// ```
    /// // .exec() is an async call so you need to await it
    /// Create.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {

        Ok(())
    }
}
