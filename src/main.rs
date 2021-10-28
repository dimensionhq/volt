/*
 *
 *    Copyright 2021 Volt Contributors
 *
 *    Licensed under the Apache License, Version 2.0 (the "License");
 *    you may not use this file except in compliance with the License.
 *    You may obtain a copy of the License at
 *
 *        http://www.apache.org/licenses/LICENSE-2.0
 *
 *    Unless required by applicable law or agreed to in writing, software
 *    distributed under the License is distributed on an "AS IS" BASIS,
 *    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *    See the License for the specific language governing permissions and
 *    limitations under the License.
 */

mod commands;
mod core;

use crate::commands::{add::*, node::*};
use crate::core::{command::Command, utils::app::App};

use clap::{Arg, ArgMatches};
use colored::Colorize;
use commands::{clean::Clean, clone::Clone, discord::Discord, init::Init};
use tracing::{self, Level};
use tracing_subscriber::filter::EnvFilter;

use std::str::FromStr;
use std::{sync::Arc, time::Instant};

pub async fn map_subcommand(matches: ArgMatches) -> miette::Result<()> {
    match matches.subcommand() {
        Some(("add", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Add::exec(app).await
        }
        Some(("clone", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Clone::exec(app).await
        }
        Some(("init", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Init::exec(app).await
        }
        Some(("clean", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Clean::exec(app).await
        }
        Some(("discord", args)) => {
            let app = Arc::new(App::initialize(args)?);
            Discord::exec(app).await
        }
        Some(("node", args)) => Node::download(args).await,
        _ => Ok(()),
    }
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or(EnvFilter::from_str("volt=info").unwrap()),
        )
        .without_time()
        .init();

    let start = Instant::now();
    let volt_help = format!(
        r#"{} {}

        Usage: {} [{}] [{}]

        Displays help information.

        Commands:
        {} add
        {} audit
        {} cache
        {} check
        {} clean"#,
        "volt".bright_green().bold(),
        "1.0.0",
        "volt".bright_green().bold(),
        "command".bright_cyan(),
        "flags".bright_blue(),
        "-".bright_magenta(),
        "-".bright_magenta(),
        "-".bright_magenta(),
        "-".bright_magenta(),
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

    let clean_usage = format!(
        "{} compress {}",
        "volt".bright_green().bold(),
        "[flags]".bright_blue(),
    );

    let clone_usage = format!(
        "{} clone {}",
        "volt".bright_green().bold(),
        "[flags]".bright_blue(),
    );

    let discord_usage = format!("{} discord", "volt".bright_green().bold());

    let app = clap::App::new("volt")
        .version("1.0.0")
        .author("XtremeDevX <xtremedevx@gmail.com>")
        .about("Manage your NPM packages")
        .override_help(volt_help.as_str())
        .arg(Arg::new("version").short('v').long("version"))
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
            clap::App::new("clone")
                .about("Clone a project and install dependencies.")
                .override_usage(clone_usage.as_str())
                .arg(
                    Arg::new("repository")
                        .about("Url of the repository to clone.")
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
            clap::App::new("clean")
                .about("Clean node_modules and reduce its size.")
                .override_usage(clean_usage.as_str()),
        )
        .subcommand(
            clap::App::new("node")
                .about("Manage node versions")
                .subcommand(
                    clap::App::new("use")
                        .about("Switch current node version")
                        .arg(Arg::new("version").about("version to use")),
                )
                .subcommand(
                    clap::App::new("remove")
                        .about("Uninstall a specified version of node")
                        .arg(
                            Arg::new("versions")
                                .multiple_values(true)
                                .about("version to remove"),
                        ),
                )
                .subcommand(
                    clap::App::new("install")
                        .about("Install one or more versions of node")
                        .arg(
                            Arg::new("versions")
                                .multiple_values(true)
                                .about("version to install"),
                        ),
                ),
        )
        .subcommand(
            clap::App::new("discord")
                .about("Join the official volt discord server.")
                .override_usage(discord_usage.as_str()),
        );

    let matches = app.get_matches();

    map_subcommand(matches).await?;

    println!("Finished in {:.2}s", start.elapsed().as_secs_f32());

    Ok(())
}
