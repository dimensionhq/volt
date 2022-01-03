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

use crate::{
    cli::{VoltCommand, VoltConfig},
    core::VERSION,
};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::sync::Arc;

/// Struct implementation for the `Update` command.
pub struct Update;

#[async_trait]
impl VoltCommand for Update {
    /// Display a help menu for the `volt update` command.
    //     fn help() -> String {
    //         format!(
    //             r#"volt {}
    //
    // Update project dependencies
    //
    // Usage: {} {} {}
    //
    // Options:
    //
    //   {} {} Output verbose messages on internal operations."#,
    //             VERSION.bright_green().bold(),
    //             "volt".bright_green().bold(),
    //             "update".bright_purple(),
    //             "file-name".white(),
    //             "--verbose".blue(),
    //             "(-v)".yellow()
    //         )
    //     }

    /// Execute the `volt update` command
    ///
    /// update project dependencies
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // update project dependencies
    /// // .exec() is an async call so you need to await it
    /// update.exec(app, vec![], vec!["--yes"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(self, config: VoltConfig) -> Result<()> {
        Ok(())
    }
}
