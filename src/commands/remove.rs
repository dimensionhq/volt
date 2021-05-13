use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;

use crate::{utils::App, __VERSION__};

use super::Command;

pub struct Remove;

#[async_trait]
impl Command for Remove {
    fn help(&self) -> String {
        format!(
            r#"volt {}
    
Removes a package from your direct dependencies.

Usage: {} {} {} {}

Options: 

  {} {} Output the version number.
  {} {} Output verbose messages on internal operations."#,
            __VERSION__.bright_green().bold(),
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

    async fn exec(&self, _app: Arc<App>, args: Vec<String>, flags: Vec<String>) -> Result<()> {
        println!("Removing packages");
        println!("Packages: {:?}", args);
        println!("Flags: {:?}", flags);
        Ok(())
    }
}
