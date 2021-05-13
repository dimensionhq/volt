use std::sync::Arc;

use async_trait::async_trait;
use colored::Colorize;

use crate::{utils::App, __VERSION__};

use super::Command;

pub struct Install;

#[async_trait]
impl Command for Install {
    fn help(&self) -> String {
        format!(
            r#"volt {}
        
Install dependencies for a project.

Usage: {} {} {}
    
Options: 
    
  {} {} Accept all prompts while installing dependencies.  
  {} {} Output verbose messages on internal operations."#,
            __VERSION__.bright_green().bold(),
            "volt".bright_green().bold(),
            "install".bright_purple(),
            "[flags]".white(),
            "--yes".blue(),
            "(-y)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    async fn exec(&self, _app: Arc<App>, _packages: &Vec<String>, _flags: &Vec<String>) {}
}
