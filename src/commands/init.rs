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

use crate::{
    classes::init_data::{InitData, License},
    prompt::prompt::{Confirm, Input, Select},
    utils::{self, get_git_config, App},
    VERSION,
};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use std::io::Write;
use std::{env, process};
use std::{fs::File, sync::Arc};

use super::Command;

/// Struct implementation for the `Init` command.
pub struct Init;

#[async_trait]
impl Command for Init {
    /// Display a help menu for the `volt init` command.
    fn help(&self) -> String {
        format!(
            r#"volt {}
    
Interactively create or update a package.json file for a project

Usage: {} {} {}
    
Options:
    
  {} {} Initialize a package.json file without any prompts.  
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "init".bright_purple(),
            "[flags]".white(),
            "--yes".blue(),
            "(-y)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt init` command
    /// Interactively create or update a package.json file for a project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// * `packages` - List of packages to add (`Vec<String>`)
    /// * `flags` - List of flags passed in through the CLI (`Vec<String>`)
    /// ## Examples
    /// ```
    /// // Initialize a new package.json file without any prompts
    /// // .exec() is an async call so you need to await it
    /// Init.exec(app, vec![], vec!["--yes"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(&self, _app: Arc<App>, _args: Vec<String>, flags: Vec<String>) -> Result<()> {
        let temp = utils::get_basename(&env::current_dir().unwrap().to_string_lossy()).to_string();
        let split: Vec<&str> = temp.split(r"\").collect::<Vec<&str>>();
        let cwd: String = split[split.len() - 1].to_string();

        let data = if flags.iter().any(|flag| flag == "-y" || flag == "--yes") {
            // Set name to current directory name
            let name = env::current_dir()
                .map(|dir| {
                    dir.file_name()
                        .map(|file_name| file_name.to_string_lossy().to_string())
                })
                .ok()
                .flatten()
                .unwrap_or_else(|| "app".to_string());

            let version = "0.1.0".to_string();

            let description = None;

            let main = "index.js".to_string();

            let author = {
                let git_user_name = get_git_config("user.name")
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| String::new());

                let git_email = get_git_config("user.email")
                    .ok()
                    .flatten()
                    .map(|email| format!("<{}>", email))
                    .unwrap_or_else(|| String::new());

                if git_user_name.is_empty() && git_email.is_empty() {
                    None
                } else {
                    Some([git_user_name, git_email].join(" "))
                }
            };

            let repository = get_git_config("remote.origin.url").ok().flatten();

            let license = License::default();

            InitData {
                name: name,
                version: version,
                description: description,
                main: main,
                repository: repository,
                author: author,
                license: license,
                private: None,
            }
        } else {
            // Get "name"
            let input: Input = Input {
                message: String::from("name"),
                default: Some(cwd),
                allow_empty: false,
            };

            let name = input.run().unwrap_or_else(|err| {
                eprintln!("{}", err);
                process::exit(1);
            });

            // Get "version"
            let input: Input = Input {
                message: String::from("version"),
                default: Some(String::from("1.0.0")),
                allow_empty: false,
            };

            let version = input.run().unwrap_or_else(|err| {
                eprintln!(
                    "{}: {}",
                    "error".bright_red().bold(),
                    err.to_string().bright_yellow()
                );
                process::exit(1);
            });

            // Get "description"
            let input: Input = Input {
                message: String::from("description"),
                default: None,
                allow_empty: true,
            };

            let description = input.run().unwrap_or_else(|err| {
                eprintln!(
                    "{}: {}",
                    "error".bright_red().bold(),
                    err.to_string().bright_yellow()
                );
                process::exit(1);
            });

            // Get "main"
            let input: Input = Input {
                message: String::from("main"),
                default: Some(String::from("index.js")),
                allow_empty: false,
            };

            let main = input.run().unwrap_or_else(|err| {
                eprintln!(
                    "{}: {}",
                    "error".bright_red().bold(),
                    err.to_string().bright_yellow()
                );
                process::exit(1);
            });

            // Get "author"
            let git_user_name = get_git_config("user.name")
                .ok()
                .flatten()
                .unwrap_or_else(|| String::new());

            let git_email = get_git_config("user.email")
                .ok()
                .flatten()
                .map(|email| format!("<{}>", email))
                .unwrap_or_else(|| String::new());

            let author;

            if git_user_name != String::new() && git_email != String::new() {
                let input: Input = Input {
                    message: String::from("author"),
                    default: Some(format!("{} {}", git_user_name, git_email)),
                    allow_empty: true,
                };
                author = input.run().unwrap_or_else(|err| {
                    eprintln!(
                        "{}: {}",
                        "error".bright_red().bold(),
                        err.to_string().bright_yellow()
                    );
                    process::exit(1);
                });
            } else {
                let input: Input = Input {
                    message: String::from("author"),
                    default: None,
                    allow_empty: true,
                };
                author = input.run().unwrap_or_else(|err| {
                    eprintln!(
                        "{}: {}",
                        "error".bright_red().bold(),
                        err.to_string().bright_yellow()
                    );
                    process::exit(1);
                });
            }

            // Get "repository"
            let input: Input = Input {
                message: String::from("repository"),
                default: None,
                allow_empty: true,
            };

            let repository = input.run().unwrap_or_else(|err| {
                eprintln!(
                    "{}: {}",
                    "error".bright_red().bold(),
                    err.to_string().bright_yellow()
                );
                process::exit(1);
            });

            let licenses: Vec<String> = License::options();

            let select = Select {
                message: String::from("License"),
                paged: true,
                selected: Some(1),
                items: licenses.clone(),
            };

            select.run().unwrap_or_else(|err| {
                eprintln!(
                    "{}: {}",
                    "error".bright_red().bold(),
                    err.to_string().bright_yellow()
                );
                process::exit(1);
            });

            let license = License::from_index(select.selected.unwrap()).unwrap();

            let input = Confirm {
                message: String::from("private"),
                default: false,
            };

            let private = input.run().unwrap_or_else(|err| {
                eprintln!(
                    "{}: {}",
                    "error".bright_red().bold(),
                    err.to_string().bright_yellow()
                );
                process::exit(1);
            });

            InitData {
                name: name,
                version: version,
                description: Some(description),
                main: main,
                repository: Some(repository),
                author: Some(author),
                license: license,
                private: Some(private),
            }
        };

        let mut file = File::create(r"package.json").unwrap();
        if let Err(error) = file.write(data.dump().as_bytes()) {
            eprintln!(
                "{} : {} {}",
                "error:".bright_red().bold(),
                "Failed To Create package.json -".bright_red(),
                error.to_string().bright_yellow().bold()
            );
            process::exit(1);
        }

        println!("{}", "Successfully Initialized package.json".bright_green());
        Ok(())
    }
}
