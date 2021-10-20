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

use crate::{core::VERSION, App, Command};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::sync::Arc;

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
    async fn exec(_app: Arc<App>) -> Result<()> {
        // let args = &app.args;

        // if args.len() <= 1 {
        //     error!("Missing Package Name!");
        //     process::exit(1);
        // }

        // let package = &args[1];

        // println!(
        //     "{}{}\n",
        //     "Download stats for ".bright_cyan().bold(),
        //     package.bright_cyan().bold()
        // );

        // // Get downloads for the past week
        // let url = format!(
        //     "https://api.npmjs.org/downloads/point/last-week/{}",
        //     package
        // );

        // let response = get(url).await.unwrap_or_else(|e| {
        //     eprintln!("{}", e.to_string());
        //     std::process::exit(1)
        // });

        // let file_contents = response.text().await.unwrap_or_else(|e| {
        //     eprintln!("{}", e.to_string());
        //     std::process::exit(1)
        // });

        // let data: Value = from_str(&file_contents).unwrap();

        // let downloads = &data["downloads"];
        // println!(
        //     "{} {}",
        //     downloads.to_string().bright_green(),
        //     "downloads in the past week!".bright_green()
        // );

        // // Get downloads for the past month
        // let url = format!(
        //     "https://api.npmjs.org/downloads/point/last-month/{}",
        //     package
        // );

        // let response = get(url).await.unwrap_or_else(|e| {
        //     eprintln!("{}", e.to_string());
        //     std::process::exit(1)
        // });

        // let file_contents = response.text().await.unwrap_or_else(|e| {
        //     eprintln!("{}", e.to_string());
        //     std::process::exit(1)
        // });

        // let data: Value = serde_json::from_str(&file_contents).unwrap();

        // let downloads = &data["downloads"];
        // println!(
        //     "{} {}",
        //     downloads.to_string().bright_green(),
        //     "downloads in the past month!".bright_green()
        // );

        // // Get downloads for the past year
        // let url = format!(
        //     "https://api.npmjs.org/downloads/point/last-year/{}",
        //     package
        // );

        // let response = get(url).await.unwrap_or_else(|e| {
        //     eprintln!("{}", e.to_string());
        //     process::exit(1)
        // });

        // let file_contents = response.text().await.unwrap_or_else(|e| {
        //     eprintln!("{}", e.to_string());
        //     process::exit(1)
        // });

        // let data: Value = serde_json::from_str(&file_contents).unwrap();

        // let downloads = &data["downloads"];
        // println!(
        //     "{} {}",
        //     downloads.to_string().bright_green(),
        //     "downloads in the past year!".bright_green()
        // );

        Ok(())
    }
}
