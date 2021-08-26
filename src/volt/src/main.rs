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

use colored::Colorize;
use std::time::Instant;
use utils::app::{App, AppFlag};
use utils::helper::CustomColorize;
use volt_core::VERSION;

#[tokio::main]
async fn main() -> miette::DiagnosticResult<()> {
    let app = App::initialize()?;
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

    // Run command
    cmd.run(app).await?;

    println!("Finished in {:.2}s", time.elapsed().as_secs_f32());

    Ok(())
}
