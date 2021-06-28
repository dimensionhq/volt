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

//! Check for outdated packages.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use volt_core::command::Command;
use volt_utils::app::App;
pub struct Publish {}
#[async_trait]
impl Command for Publish {
    fn help() -> String {
        todo!()
    }

    /// Execute the `volt outdated` command
    ///
    /// Check for outdated packages
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Check for outdated packages
    /// // .exec() is an async call so you need to await it
    /// Publish.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        Ok(())
    }
}
