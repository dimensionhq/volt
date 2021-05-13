use std::sync::Arc;

use async_trait::async_trait;
use colored::Colorize;

use crate::{utils::App, __VERSION__};

use super::Command;

pub struct Help;

#[async_trait]
impl Command for Help {
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
            __VERSION__.bright_green().bold(),
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

    async fn exec(&self, _app: Arc<App>, _args: Vec<String>, _flags: Vec<String>) {
        println!("{}", self.help());
    }
}
