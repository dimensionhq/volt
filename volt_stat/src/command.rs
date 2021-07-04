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

//! Display stats on a specific package

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use reqwest::get;
use serde_json::Value;
use volt_core::{command::Command, VERSION};
use volt_utils::app::App;

/// Struct implementation for the `stat` command.
pub struct Stat;

#[async_trait]
impl Command for Stat {
    /// Display a help menu for the `volt stat` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
Displays statistics on a specific package.

Usage: {} {} {}"#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "stat".bright_purple(),
            "[package]".white(),
        )
    }

    /// Execute the `volt stat` command
    ///
    /// Displays stats on a specific package
    /// ## Examples
    /// ```
    /// // .exec() is an async call so you need to await it
    /// Create.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        let args = &app.args;
        let package = &args[1];

        println!("{}\n", package.bright_cyan().bold());

        // Get downloads for the past week
        let url = format!(
            "https://api.npmjs.org/downloads/point/last-week/{}",
            package
        );

        let response = get(url).await.unwrap_or_else(|e| {
            eprintln!("{}", e.to_string());
            std::process::exit(1)
        });

        let file_contents = response.text().await.unwrap_or_else(|e| {
            eprintln!("{}", e.to_string());
            std::process::exit(1)
        });

        let data: Value = serde_json::from_str(&file_contents)?;

        let downloads = &data["downloads"];
        println!(
            "{} {}",
            downloads.to_string().bright_green(),
            "downloads in the past week!".bright_green()
        );

        // Get downloads for the past month
        let url = format!(
            "https://api.npmjs.org/downloads/point/last-month/{}",
            package
        );

        let response = get(url).await.unwrap_or_else(|e| {
            eprintln!("{}", e.to_string());
            std::process::exit(1)
        });

        let file_contents = response.text().await.unwrap_or_else(|e| {
            eprintln!("{}", e.to_string());
            std::process::exit(1)
        });

        let data: Value = serde_json::from_str(&file_contents)?;

        let downloads = &data["downloads"];
        println!(
            "{} {}",
            downloads.to_string().bright_green(),
            "downloads in the past month!".bright_green()
        );

        // Get downloads for the past year
        let url = format!(
            "https://api.npmjs.org/downloads/point/last-year/{}",
            package
        );

        let response = get(url).await.unwrap_or_else(|e| {
            eprintln!("{}", e.to_string());
            std::process::exit(1)
        });

        let file_contents = response.text().await.unwrap_or_else(|e| {
            eprintln!("{}", e.to_string());
            std::process::exit(1)
        });

        let data: Value = serde_json::from_str(&file_contents)?;

        let downloads = &data["downloads"];
        println!(
            "{} {}",
            downloads.to_string().bright_green(),
            "downloads in the past year!".bright_green()
        );

        Ok(())
    }
}
