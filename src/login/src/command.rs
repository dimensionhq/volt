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

//! Login for a package.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use volt_core::command::Command;
use utils::app::App;
pub struct Login {}
#[async_trait]
impl Command for Login {
    fn help() -> String {
        todo!()
    }

    /// Execute the `volt search` command
    ///
    /// Login for a package
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Login for a package
    /// // .exec() is an async call so you need to await it
    /// Login.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        Ok(())
    }
}