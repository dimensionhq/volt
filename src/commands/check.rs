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

use crate::{core::VERSION, App, Command};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::sync::Arc;

/// Struct implementation for the `Check` command.
pub struct Check;

#[async_trait]
impl Command for Check {
    /// Display a help menu for the `volt check` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Checks for errors.

Usage: {} {}

Options: 

  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "check".bright_purple(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt Check` command
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
