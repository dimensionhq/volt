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

use crate::{core::VERSION, App, Command};

use async_trait::async_trait;
use colored::Colorize;
use isahc::AsyncReadResponseExt;
use miette::Result;
use prettytable::{cell, row, Table};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => (s[..idx].to_owned() + "..."),
    }
}

pub struct Search {}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SearchData {
    pub name: String,
    pub version: String,
    pub description: String,
}

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
        let query = app.args.value_of("query").unwrap();

        let response =
            isahc::get_async(format!("https://npmjs.com/search/suggestions?q={}", query))
                .await
                .unwrap()
                .text()
                .await
                .unwrap();

        let s: Vec<SearchData> = serde_json::from_str(&response).unwrap();

        let mut table = Table::new();

        table.add_row(row![
            "Name".bright_green().bold(),
            "Version".bright_green().bold(),
            "Description".bright_green().bold()
        ]);

        for i in s.iter() {
            table.add_row(row![i.name, i.version, truncate(&i.description, 35)]);
        }

        table.printstd();

        Ok(())
    }
}
