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

//! Search for a package.
use std::sync::Arc;
use crate::search::SearchData;
use anyhow::Result;
use async_trait::async_trait;
use chttp::ResponseExt;
use colored::Colorize;
use prettytable::row;
// use search::SearchResp;
use volt_core::{VERSION, command::Command};
use volt_utils::app::App;
use prettytable::{Table, cell};
fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => (s[..idx].to_owned() + "...").to_string(),
    }
}

pub struct Search {}
#[async_trait]
impl Command for Search {
    fn help() -> String {
        format!(
            r#"volt {}

Searches for a package 

Usage: {} {} {} {}

Options: 

  {} {} Output the version number.
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "remove".bright_purple(),
            "[packages]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt search` command
    ///
    /// Search for a package
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Search for a package
    /// // .exec() is an async call so you need to await it
    /// Search.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        if app.args.len() >= 2 {
            let package_name = &app.args[1];
        
            let response = chttp::get_async(format!("https://www.npmjs.com/search/suggestions?q={}", package_name))
                .await
                .unwrap_or_else(|_| {
                    println!("{}: package does not exist", "error".bright_red(),);
                    std::process::exit(1);
                })
                .text_async()
                .await
                .unwrap_or_else(|_| {
                    println!("{}: package does not exist", "error".bright_red());
                    std::process::exit(1);
                });
                let s: Vec<SearchData> = serde_json::from_str(&response)
                    .unwrap_or_else(|e| {
                        println!(
                            "{}: failed to parse response from server {}",
                            "error".bright_red(),
                            e.to_string().bright_red(),
                        );
                        
                        std::process::exit(1);
                    });
            


                    let mut table = Table::new();
                    table.add_row(row!["Name".green().bold(), "Version".green().bold(), "Description".green().bold()]);
                    for i in s.iter(){
                        
                        table.add_row(row![i.name, i.version, truncate(&i.description, 20)]);
                    }
                    table.printstd();
                
            // let u: SearchResp = s;
            // panic!("{:#?}", s);
        }
        Ok(())
    }

}
