// Modules
mod classes;
mod commands;
mod model;
mod prompt;
mod utils;

// Imports
use commands::AppCommand;
use utils::{get_arguments, initialize};

// Constants
const __VERSION__: &str = "v1.0.0";

#[tokio::main]
async fn main() {
    let args: Vec<String> = initialize();

    let (flags, args) = get_arguments(&args);

    let app_cmd = AppCommand::current().unwrap_or(AppCommand::Help);
    let cmd = app_cmd.command();

    if flags.iter().any(|flag| flag == "--help") {
        println!("{}", cmd.help());
        std::process::exit(0);
    }

    cmd.exec(&args, &flags).await
}
