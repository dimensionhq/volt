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

use crate::{core::VERSION, App, Command};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::sync::Arc;

pub struct Discord {}

#[async_trait]
impl Command for Discord {
    fn help() -> String {
        format!(
            r#"volt {}

Join the official volt discord server.

Usage: {} {}"#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "discord".bright_purple(),
        )
    }

    /// Execute the `volt discord` command
    ///
    /// Join the official volt discord server.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Opens a link to the official volt discord server.
    /// // .exec() is an async call so you need to await it
    /// Discord.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        match webbrowser::open("https://discord.gg/fY7BMcrcYr") {
            Ok(_) => {
                println!("Successfully opened an invite to the official {} server on your default browser.", "discord".truecolor(88, 101, 242).bold());
            }
            Err(_err) => {
                println!("Failed to open an invite to the official {} server on your default browser.\nFeel free to join using this link instead: {}", "discord".truecolor(88, 101, 242).bold(), "https://discord.gg/fY7BMcrcYr".bright_purple().underline());
            }
        };

        Ok(())
    }
}
