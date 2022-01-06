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

//! Clone and setup a repository from Github.

use crate::cli::{VoltCommand, VoltConfig};

use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use miette::Result;
use std::{process, sync::Arc};

/// Clone a project and setup a project from a repository
#[derive(Debug, Parser)]
pub struct Clone {
    /// URL of the repository
    repository: String,
}

#[async_trait]
impl VoltCommand for Clone {
    /// Execute the `volt clone` command
    ///
    /// Clone and setup a repository from Github
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Clone the react repository (https://github.com/facebook/react)
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(self, _: VoltConfig) -> miette::Result<()> {
        let exit_code = process::Command::new("cmd")
            .arg(format!("/C git clone {} --depth=1", self.repository).as_str())
            .status()
            .unwrap();

        if exit_code.success() {
            process::Command::new("volt")
                .arg("install")
                .spawn()
                .unwrap();
        } else {
        }

        Ok(())
    }
}
