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
#![allow(unused)]

mod commands;
mod core;

use crate::core::{command::Command, utils::app::App};

use clap::{Arg, ArgMatches};
use colored::Colorize;
use commands::{
    add::Add, clean::Clean, clone::Clone, discord::Discord, info::Info, init::Init, login::Login,
    node::Node, run::Run, search::Search,
};
use tracing::{self, Level};
use tracing_subscriber::filter::EnvFilter;

use std::{str::FromStr, sync::Arc, time::Instant};

#[tokio::main]
async fn main() -> miette::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::from_str("volt=info").unwrap()),
        )
        .without_time()
        .init();

    let start = Instant::now();

    let help_texts = help_texts();
    let app = setup_app(&help_texts);
    let matches = app.get_matches();
    run_subcommand(matches).await?;

    println!("Finished in {:.2}s", start.elapsed().as_secs_f32());
    std::process::exit(0);

    Ok(())
}

fn help_texts() -> HelpTexts {
    HelpTexts {
        volt_help: format!(
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
        ),
        add_usage: format!(
            "{} add {}",
            "volt".bright_green().bold(),
            "<package-name>".bright_blue()
        ),
        init_usage: format!(
            "{} init {}",
            "volt".bright_green().bold(),
            "[flags]".bright_blue(),
        ),
        clean_usage: format!(
            "{} clean {}",
            "volt".bright_green().bold(),
            "[flags]".bright_blue(),
        ),
        clone_usage: format!(
            "{} clone {}",
            "volt".bright_green().bold(),
            "[flags]".bright_blue(),
        ),
        search_usage: format!(
            "{} search {} {}",
            "volt".bright_green().bold(),
            "<query>".bright_cyan().bold(),
            "[flags]".bright_blue(),
        ),
        login_usage: format!(
            "{} login {}",
            "volt".bright_green().bold(),
            "[flags]".bright_blue(),
        ),
        run_usage: format!(
            "{} run {}",
            "volt".bright_green().bold(),
            "[flags]".bright_blue()
        ),
        info_usage: format!(
            "{} info {}",
            "volt".bright_green().bold(),
            "[flags]".bright_blue(),
        ),
        discord_usage: format!("{} discord", "volt".bright_green().bold()),
    }
}

struct HelpTexts {
    volt_help: String,
    add_usage: String,
    init_usage: String,
    clean_usage: String,
    clone_usage: String,
    search_usage: String,
    login_usage: String,
    run_usage: String,
    info_usage: String,
    discord_usage: String,
}

fn setup_app(help_texts: &HelpTexts) -> clap::App<'_> {
    let app = clap::App::new("volt")
        .version("1.0.0")
        .author("XtremeDevX <xtremedevx@gmail.com>")
        .about("Manage your NPM packages")
        .override_help(help_texts.volt_help.as_str())
        .arg(Arg::new("version").short('v').long("version"))
        .subcommand(
            clap::App::new("add")
                .about("Add a package to the dependencies for your project.")
                .override_usage(help_texts.add_usage.as_str())
                .arg(
                    Arg::new("package-names")
                        .about("Packages to add to the dependencies for your project.")
                        .multiple_values(true)
                        .required(true),
                ),
        )
        .subcommand(
            clap::App::new("clean")
                .about("Optimizes your node_modules by removing redundant files and folders.")
                .override_usage(help_texts.clean_usage.as_str())
                .arg(Arg::new("remove-licenses").long("remove-licenses")),
        )
        .subcommand(
            clap::App::new("clone")
                .about("Clone a project and install dependencies.")
                .override_usage(help_texts.clone_usage.as_str())
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
                .override_usage(help_texts.init_usage.as_str())
                .arg(Arg::new("yes").short('y').about("Use default options")),
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
                .override_usage(help_texts.discord_usage.as_str()),
        )
        .subcommand(
            clap::App::new("search")
                .about("Search for a package.")
                .override_usage(help_texts.search_usage.as_str())
                .arg(
                    Arg::new("query")
                        .about("The search query string")
                        .required(true),
                ),
        )
        .subcommand(
            clap::App::new("run")
                .about("Run a defined package script.")
                .override_usage(help_texts.run_usage.as_str())
                .arg(
                    Arg::new("script-name")
                        .about("Name of the script to be run")
                        .required(true),
                ),
        )
        .subcommand(
            clap::App::new("info")
                .about("Display information about a package.")
                .override_help("todo")
                .override_usage(help_texts.info_usage.as_str()),
        )
        .subcommand(
            clap::App::new("login")
                .about("Login to the npm registry.")
                .override_help("todo")
                .override_usage(help_texts.login_usage.as_str()),
        );

    app
}

async fn run_subcommand(matches: ArgMatches) -> miette::Result<()> {
    match matches.subcommand() {
        Some(("node", args)) => Node::download(args).await,
        Some((subcommand, args)) => {
            let app = Arc::new(App::initialize(args)?);
            match subcommand {
                "add" => Add::exec(app).await,
                "clone" => Clone::exec(app).await,
                "init" => Init::exec(app).await,
                "clean" => Clean::exec(app).await,
                "discord" => Discord::exec(app).await,
                "search" => Search::exec(app).await,
                "login" => Login::exec(app).await,
                "run" => Run::exec(app).await,
                "info" => Info::exec(app).await,
                _ => Ok(()),
            }
        }
        _ => Ok(()),
    }
}
