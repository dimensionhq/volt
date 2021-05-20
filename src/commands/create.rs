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
use std::process;
use crate::prompt::prompt::{Confirm, Input, Select};
use crate::classes::create_templates::{Template, CreateTemplate};

// Crate Level Imports
use crate::utils::App;
use crate::VERSION;

// Super Imports
use super::Command;

/// Struct implementation for the `Remove` command.
pub struct Create;

#[async_trait]
impl Command for Create {
    /// Display a help menu for the `volt create` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Creates a project from a template.

Usage: {} {} {} {}

Options: 

  {} {} Output the version number.
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "create".bright_purple(),
            "[template]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt create` command
    ///
    /// Removes a package from your direct dependencies.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `template` - Template to create with
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Remove a package from your direct dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Create.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        let args = app.args.clone();
        println!("{:#?}", args);       

        let templates: Vec<String> = Template::options();

        let select = Select {
            message: String::from("Template"),
            paged: true,
            selected: Some(1),
            items: templates.clone(),
        };

        select.run().unwrap_or_else(|err| {
            eprintln!(
                "{}: {}",
                "error".bright_red().bold(),
                err.to_string().bright_yellow()
            );
            process::exit(1);
        });

        let template = Template::from_index(select.selected.unwrap()).unwrap();

        println!("template: {}", template);

        Ok(())
    }
}
