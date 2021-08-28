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

pub mod commands;

use std::time::Instant;

use clap::{
    App,
    Arg,
};
use colored::Colorize;

// use colored::Colorize;
// use std::time::Instant;
// use utils::app::{App, AppFlag};
// use utils::helper::CustomColorize;
// use volt_core::VERSION;

#[tokio::main]
async fn main() -> miette::DiagnosticResult<()>
{
    let volt_help = format!(
        r#"{} {}

Usage: {} [{}] [{}]

Displays help information.

Commands:
  {} add"#,
        "volt".bright_green().bold(),
        "1.0.0",
        "volt".bright_green().bold(),
        "command".bright_cyan(),
        "flags".bright_blue(),
        "-".bright_magenta()
    );

    let app = App::new("volt")
        .version("1.0.0")
        .author("XtremeDevX <xtremedevx@gmail.com>")
        .about("Manage your NPM packages")
        .override_help(volt_help.as_str())
        .subcommand(
            App::new("add")
                .about("Add a package to the dependencies for your project.")
                .arg(
                    Arg::new("package-name")
                        .about("Package to add to the dependencies for your project.")
                        .index(1)
                        .required(true),
                ),
        );

    let matches = app.get_matches();

    println!("{:?}", matches);
    Ok(())
}
