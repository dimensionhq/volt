use colored::Colorize;

use crate::__VERSION__;

use super::Command;

pub struct Add;

impl Command for Add {
    fn help(&self) -> String {
        format!(
            r#"volt {}
    
    Add a package to your dependencies for your project.
    
    Usage: {} {} {} {}
    
    Options: 
        
      {} {} Output the version number.
      {} {} Output verbose messages on internal operations.
      {} {} Disable progress bar."#,
            __VERSION__.bright_green().bold(),
            "volt".bright_green().bold(),
            "add".bright_purple(),
            "[packages]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    fn exec(&self, args: &Vec<String>, flags: &Vec<String>) {
        println!("Adding packages");
        println!("Packages: {:?}", args);
        println!("Flags: {:?}", flags);
    }
}
