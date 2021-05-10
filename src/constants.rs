use colored::Colorize;

const __VERSION__: &str = "v1.0.0";

pub fn about() {
    let about: String = format!(
        r#"{}

Usage: {} {} [options]

{} {} - Install all dependencies for a project.
{} {} - Interactively create or update a package.json file for a project.
{} {} - Add a dependency to a project.
{} {} - Remove a dependency from the package.json file for a project."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
        "volt".bright_green().bold(),
        "command".bright_blue(),
        "*".bright_magenta().bold(),
        "install".bright_blue(),
        "*".bright_magenta().bold(),
        "init".bright_blue(),
        "*".bright_magenta().bold(),
        "add".bright_blue(),
        "*".bright_magenta().bold(),
        "remove".bright_blue()
    );

    println!("{}", about);
    std::process::exit(0);
}

pub fn help() {
    let help: String = format!(
        r#"{}

Displays help information.

Usage: {} {} {}

Commands:

  {} {} - Install all dependencies for a project.
  {} {} - Interactively create or update a package.json file for a project.
  {} {} - Add a dependency to a project.
  {} {} - Remove a dependency from the package.json file for a project."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
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
    );

    println!("{}", help);
    std::process::exit(0);
}

pub fn init_help() {
    let init = format!(
        r#"{}

Interactively create or update a package.json file for a project

Usage: {} {} {}
    
Options:
    
  {} {} Initialize a package.json file without any prompts.  
  {} {} Output verbose messages on internal operations."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
        "volt".bright_green().bold(),
        "init".bright_purple(),
        "[flags]".white(),
        "--yes".blue(),
        "(-y)".yellow(),
        "--verbose".blue(),
        "(-v)".yellow()
    );
    println!("{}", init);
    std::process::exit(0);
}

pub fn install_help() {
    let install = format!(
        r#"{}
    
Install dependencies for a project.

Usage: {} {} {}
    
Options: 
    
  {} {} Accept all prompts while installing dependencies.  
  {} {} Output verbose messages on internal operations."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
        "volt".bright_green().bold(),
        "install".bright_purple(),
        "[flags]".white(),
        "--yes".blue(),
        "(-y)".yellow(),
        "--verbose".blue(),
        "(-v)".yellow()
    );
    println!("{}", install);
    std::process::exit(0);
}

pub fn add_help() {
    let add = format!(
        r#"{}

Add a package to your dependencies for your project.

Usage: {} {} {} {}

Options: 
    
  {} {} Output the version number.
  {} {} Output verbose messages on internal operations.
  {} {} Disable progress bar."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
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
    );
    println!("{}", add);
    std::process::exit(0);
}

pub fn remove_help() {
    let remove = format!(
        r#"{}

Removes a package from your direct dependencies.

Usage: {} {} {} {}

Options: 

  {} {} Output the version number.
  {} {} Output verbose messages on internal operations."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
        "volt".bright_green().bold(),
        "remove".bright_purple(),
        "[packages]".white(),
        "[flags]".white(),
        "--version".blue(),
        "(-ver)".yellow(),
        "--verbose".blue(),
        "(-v)".yellow()
    );
    println!("{}", remove);
    std::process::exit(0);
}

pub fn add_error() {
    let add_error = format!(
        r#"{}

{} Missing list of packages to add to your project.
    
{} Use {} for more information about this command."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
        "error".bright_red(),
        "info".bright_blue(),
        "volt add --help".bright_green()
    );
    println!("{}", add_error);
    std::process::exit(1);
}

pub fn remove_error() {
    let remove = format!(
        r#"{}

{} Not enough arguments, expected at least 1.
    
{} Use {} for more information about this command."#,
        format!("volt {}", __VERSION__.bright_green().bold()),
        "error".bright_red(),
        "info".bright_blue(),
        "volt remove --help".bright_green()
    );
    println!("{}", remove);
    std::process::exit(1);
}
