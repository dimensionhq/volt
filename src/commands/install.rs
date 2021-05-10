use colored::Colorize;

use crate::__VERSION__;

use super::Command;

pub struct Install;

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

    fn exec(&self, _args: &Vec<String>, flags: &Vec<String>) {
        println!("Installing packages");
        println!("Flags: {:?}", flags);
    }
}
