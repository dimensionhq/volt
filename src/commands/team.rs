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

use async_trait::async_trait;
use miette::Result;

use crate::cli::{VoltCommand, VoltConfig};

pub struct Team {}

#[async_trait]
impl VoltCommand for Team {
    /// Execute the `volt outdated` command
    ///
    /// Check for outdated packages
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Check for outdated packages
    /// // .exec() is an async call so you need to await it
    /// Team.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        Ok(())
    }
}
