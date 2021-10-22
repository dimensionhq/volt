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

//! Set a configuration option.

use crate::{App, Command};

use async_trait::async_trait;
use miette::Result;

use std::sync::Arc;

pub struct Set {}

#[async_trait]
impl Command for Set {
    fn help() -> String {
        todo!()
    }

    /// Execute the `volt set` command
    ///
    /// Set a configuration option.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Set a configuration option.
    /// // .exec() is an async call so you need to await it
    /// Set.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        Ok(())
    }
}
