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
mod core;
use std::{sync::Arc, time::Instant};

use crate::core::command::Command;
use crate::core::utils::app::App;
use clap::{Arg, ArgMatches};
use colored::Colorize;
use commands::{clean::Clean, init::Init};

use crate::commands::add::*;

pub async fn map_subcommand(matches: ArgMatches) -> miette::Result<()> {
    match matches.subcommand() {
        Some(("add", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Add::exec(app).await
        }
        Some(("init", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Init::exec(app).await
        }
        Some(("compress", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Clean::exec(app).await
        }
        _ => Ok(()),
    }
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    let start = Instant::now();
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

    let init_usage = format!(
        "{} init {}",
        "volt".bright_green().bold(),
        "[flags]".bright_blue(),
    );

    let compress_usage = format!(
        "{} compress {}",
        "volt".bright_green().bold(),
        "[flags]".bright_blue(),
    );

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
        )
        .subcommand(
            clap::App::new("init")
                .about("Interactively create and edit your package.json file.")
                .override_usage(init_usage.as_str())
                .arg(Arg::new("yes").short('y').about("Use default options")),
        )
        .subcommand(
            clap::App::new("compress")
                .about("Interactively create and edit your package.json file.")
                .override_usage(compress_usage.as_str()),
        );

    let matches = app.get_matches();

    map_subcommand(matches).await?;

    println!("Finished in {:.2}s", start.elapsed().as_secs_f32());

    Ok(())
}
