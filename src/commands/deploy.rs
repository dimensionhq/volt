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

/// Struct implementation for the `Deploy` command.
pub struct Deploy;

#[async_trait]
impl Command for Deploy {
    /// Display a help menu for the `volt deploy` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Deploys your commit to Github.

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

    /// Execute the `volt deploy` command
    ///
    /// Removes a package from your direct dependencies.
    /// ## Arguments
    /// * `commit_msg` - Name of the commit message.
    /// ## Examples
    /// ```
    /// // .exec() is an async call so you need to await it
    /// Create.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        let args: Vec<String> = app.args.clone();
        let commit_msg = &args[0];
        println!("{}", commit_msg);        
        std::process::Command::new("git").args(&["add", "."]).output().expect("Failed to add");
        std::process::Command::new("git").args(&["commit", "-m", commit_msg.as_str()]).output().expect("Failed to commit");
        // std::process::Command::new("git").args(&["push"]).output().expect("Failed to push");
        Ok(())
    }
}
