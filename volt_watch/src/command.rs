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

//! Handle an unknown command (can be listed in scripts).

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use volt_core::command::Command;
use volt_utils::app::App;
use volt_utils::package::PackageJson;

pub struct Watch {}

#[async_trait]
impl Command for Watch {
    fn help() -> String {
        todo!()
    }

    /// Execute the `volt watch` command
    ///
    /// Execute a watch command
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Scan for errors / a specific error in the code and fix it
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        println!("{}", "Scanning for errors".bright_cyan());

        Ok(())
    }
}
