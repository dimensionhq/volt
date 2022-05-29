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

//! Fix common errors in the package.json file

use crate::cli::{VoltCommand, VoltConfig};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

/// Struct implementation for the `Deploy` command.
pub struct Fix;

#[async_trait]
impl VoltCommand for Fix {
    /// Execute the `volt fix` command
    ///
    /// Removes a package from your direct dependencies.
    /// ## Examples
    /// ```
    /// // .exec() is an async call so you need to await it
    /// Create.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        println!("{}", "Scanning for errors".bright_cyan());

        Ok(())
    }
}
