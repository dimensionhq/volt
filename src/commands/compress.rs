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

//! Compress node_modules into node_modules.pack.

use std::fs::remove_file;
use std::io::{Read, Write};
use std::sync::Arc;

use crate::App;
use crate::{core::VERSION, Command};
use async_trait::async_trait;
use colored::Colorize;
use miette::Result;
pub struct Compress {}

#[async_trait]
impl Command for Compress {
    /// Display a help menu for the `volt compress` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Compress node_modules into node_modules.pack.
Usage: {} {} {} {}
Options: 
    
  {} {} Output verbose messages on internal operations.
  {} {} Disable progress bar."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "clone".bright_purple(),
            "[repository]".white(),
            "[flags]".white(),
            "--verbose".blue(),
            "(-v)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    /// Execute the `volt compress` command
    ///
    /// Compress node_modules into node_modules.pack.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Compress node_modules into node_modules.pack
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        let readme_patterns = vec!["readme.md", "changelog.md", "history.md"];

        for entry in jwalk::WalkDir::new("node_modules") {
            let path = entry.unwrap().path();

            let mut contents = String::new();

            if path.is_file() {
                let mut file = std::fs::File::open(&path).unwrap_or_else(|err| {
                    println!("{}", err);
                    std::process::exit(1);
                });

                file.read_to_string(&mut contents).unwrap();

                let extension = path.extension();

                if extension.is_some() {
                    let extension = extension.unwrap();

                    if extension == "md" {
                        let file_name = path.file_name().unwrap();

                        if readme_patterns
                            .contains(&file_name.to_str().unwrap().to_lowercase().as_str())
                        {
                            remove_file(path).unwrap();
                        };
                    } else if extension == "json" {
                        let minified = minifier::json::minify(&contents);

                        file.write(minified.as_bytes()).unwrap();
                    }
                };
            }
        }

        Ok(())
    }
}
