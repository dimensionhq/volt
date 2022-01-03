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

use crate::{
    cli::{VoltCommand, VoltConfig},
    core::VERSION,
    App, Command,
};

use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Attribute, Cell, Color, ContentArrangement,
    Table,
};
use isahc::AsyncReadResponseExt;
use miette::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Objects {
    objects: Vec<SearchResults>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResults {
    package: SearchResult,
}

#[derive(Serialize, Deserialize)]
pub struct SearchResult {
    name: String,
    version: String,
    description: String,
}

/// Searches for a package
#[derive(Debug, Parser)]
pub struct Search {
    /// Search query
    query: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SearchData {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

#[async_trait]
impl VoltCommand for Search {
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
    async fn exec(self, app: VoltConfig) -> Result<()> {
        let response = isahc::get_async(format!(
            "https://registry.npmjs.org/-/v1/search?text={}&popularity=1.0",
            self.query
        ))
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

        let s: Objects = serde_json::from_str(&response).unwrap();

        let mut table = Table::new();

        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth);

        table.set_header(vec![
            Cell::new("Name")
                .fg(Color::Green)
                .add_attribute(Attribute::Bold),
            Cell::new("Version")
                .fg(Color::Blue)
                .add_attribute(Attribute::Bold),
            Cell::new("Description")
                .fg(Color::Yellow)
                .add_attribute(Attribute::Bold),
        ]);

        for i in &s.objects {
            let mut description: String = i.package.description.clone();

            if i.package.description.len() > 150 {
                description.truncate(147);
                description = format!("{}...", description);
            }

            table.add_row(vec![
                Cell::new(&i.package.name),
                Cell::new(&i.package.version),
                Cell::new(description),
            ]);
        }

        println!("{}", table);

        Ok(())
    }
}
