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
use crate::search::SearchData;
use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use isahc::AsyncReadResponseExt;

use prettytable::row;
use std::sync::Arc;
// use search::SearchResp;
use prettytable::{cell, Table};
use utils::{app::App, error};
use volt_core::{command::Command, VERSION};

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

            let response = isahc::get_async(format!(
                "http://www.npmjs.com/search/suggestions?q={}",
                package_name
            ))
            .await
            .unwrap_or_else(|_| {
                error!("package does not exist");
                std::process::exit(1);
            })
            .text()
            .await
            .unwrap_or_else(|_| {
                error!("package does not exist");
                std::process::exit(1);
            });
            let s: Vec<SearchData> = serde_json::from_str(&response).unwrap_or_else(|e| {
                error!("failed to parse response from server {}", e.to_string());

                std::process::exit(1);
            });

            let mut table = Table::new();
            table.add_row(row![
                "Name".green().bold(),
                "Version".green().bold(),
                "Description".green().bold()
            ]);
            for i in s.iter() {
                table.add_row(row![i.name, i.version, truncate(&i.description, 35)]);
            }
            table.printstd();

            // let u: SearchResp = s;
            // panic!("{:#?}", s);
        }
        Ok(())
    }
}
