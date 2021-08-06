/*
    Copyright 2021 Volt Contributors

    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at

        http://www.apache.org/licenses/LICENSE-2.0

    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

mod commands;

use crate::commands::AppCommand;

use anyhow::Result;
use colored::Colorize;
use tokio::time::Instant;
use utils::app::{App, AppFlag};
use utils::error;
use utils::helper::CustomColorize;
use volt_core::VERSION;

#[tokio::main]
async fn main() {
    if let Err(err) = try_main().await {
        error!("{}", &err);

        let err_chain = err.chain().skip(1);
        if err_chain.clone().next().is_some() {
            error!("{}", "\nCaused by:".caused_by_style());
        }

        err_chain.for_each(|e| eprintln!("{}", e));

        println!(
            "Need help? Check out {} for help",
            "https://voltpkg.com/support".truecolor(155, 255, 171)
        );

        std::process::exit(1);
    }
}

async fn try_main() -> Result<()> {
    let app = App::initialize().unwrap();
    let cmd = AppCommand::current().unwrap_or(AppCommand::Script); // Default command is help

    if app.has_flag(AppFlag::Help) {
        // Display help message
        println!("{}", cmd.help());
        return Ok(());
    }

    if app.has_flag(AppFlag::Version) {
        // Display version
        println!("volt v{}{}", "::".bright_magenta(), VERSION.success_style());
        return Ok(());
    }

    let time = Instant::now();
    cmd.run(app).await?;
    println!("Finished in {:.2}s", time.elapsed().as_secs_f32());

    Ok(())
}
