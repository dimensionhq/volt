use async_trait::async_trait;
use colored::Colorize;
use miette::Result;
use std::sync::Arc;

use crate::core::{command::Command, utils::app::App, VERSION};

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
