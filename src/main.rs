// Modules
mod classes;
mod commands;
mod model;
mod prompt;
mod traits;
mod utils;

// Imports
use commands::AppCommand;
use std::sync::Arc;
use utils::{get_arguments, initialize};

// Constants
const __VERSION__: &str = "v1.0.0";

#[tokio::main]
async fn main() {
    let (app, args) = initialize();

    let (flags, args) = get_arguments(&args);

    let app_cmd = AppCommand::current().unwrap_or(AppCommand::Help);
    let cmd = app_cmd.command();

    if flags.iter().any(|flag| flag == "--help") {
        println!("{}", cmd.help());
        std::process::exit(0);
    }

    cmd.exec(Arc::new(app), args, flags).await
}
