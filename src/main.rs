// Modules
mod commands;
mod constants;
mod prompt;
mod utils;

// Imports
use utils::{display_help, get_arguments, handle_invalid_command, initialize};

// Constants
const __VERSION__: &str = "v1.0.0";

fn main() {
    // Initialize And Display Help Menu
    let args: Vec<String> = initialize();

    let command = display_help(&args);

    let (flags, packages) = get_arguments(&args);

    match command.as_str() {
        "init" => commands::init::init(&flags),
        "install" => commands::install::install(&flags),
        "remove" => commands::remove::remove(&flags, &packages),
        "add" => commands::add::add(&flags, &packages),
        &_ => {
            handle_invalid_command(command.as_str());
        }
    }
}
