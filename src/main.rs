// Modules
mod classes;
mod commands;
mod model;
mod prompt;
mod utils;

// Imports
use anyhow::Result;
use colored::Colorize;
use commands::AppCommand;
use std::sync::Arc;
use utils::{get_arguments, initialize};

// Constants
const __VERSION__: &str = "v1.0.0";

#[tokio::main]
async fn main() {
    if let Err(err) = try_main().await {
        eprintln!("{} {}", "error".red().bold(), err);
        let err_chain = err.chain().skip(1);
        if err_chain.clone().next().is_some() {
            eprintln!("{}", "\nCaused by:".italic().truecolor(190, 190, 190));
        }
        err_chain.for_each(|cause| eprintln!(" - {}", cause.to_string().truecolor(190, 190, 190)));
        #[cfg(not(debug_assertions))]
        eprintln!(
            "\nIf the problem persists, please submit an issue on the Github repository.\n{}",
            "https://github.com/voltpkg/volt/issues/new".underline()
        );
        std::process::exit(1);
    }
}

async fn try_main() -> Result<()> {
    let (app, args) = initialize();

    let (flags, args) = get_arguments(&args);

    let app_cmd = AppCommand::current().unwrap_or(AppCommand::Help); // Default command is help
    let cmd = app_cmd.command();

    if flags.iter().any(|flag| flag == "--help") {
        println!("{}", cmd.help());
        return Ok(());
    }

    cmd.exec(Arc::new(app), args, flags).await
}
