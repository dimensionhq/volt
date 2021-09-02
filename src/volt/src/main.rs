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

use std::sync::Arc;

use clap::{Arg, ArgMatches};
use colored::Colorize;
use utils::app::App;
use volt_core::command::Command;

use crate::commands::add::*;
use crate::commands::audit::*;

pub async fn map_subcommand(matches: ArgMatches) -> miette::DiagnosticResult<()> {
    match matches.subcommand() {
        Some(("add", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Add::exec(app).await
        }
        _ => Ok(()),
    }
}

#[tokio::main]
async fn main() -> miette::DiagnosticResult<()> {
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

    let add_usage = format!(
        "{} add {}",
        "volt".bright_green().bold(),
        "<package-name>".bright_blue()
    );

    // https://docs.rs/clap/2.33.3/clap/struct.App.html?search=#method.usage

    let app = clap::App::new("volt")
        .version("1.0.0")
        .author("XtremeDevX <xtremedevx@gmail.com>")
        .about("Manage your NPM packages")
        .override_help(volt_help.as_str())
        .subcommand(
            clap::App::new("add")
                .about("Add a package to the dependencies for your project.")
                .override_usage(add_usage.as_str())
                .arg(
                    Arg::new("package-names")
                        .about("Packages to add to the dependencies for your project.")
                        .multiple_values(true)
                        .required(true),
                ),
        );

    // let matches = app.get_matches();
    let matches = app.try_get_matches();
    match matches {
        Ok(_) => {
            println!("Is ok!");
        }
        Err(_) => {
            println!("Failed!");
            for arg in std::env::args().skip(1) {
                println!("{:?}", arg);
            }
        }
    }
    // app.get_arguments()

    // map_subcommand(matches).await?;

    Ok(())
}
