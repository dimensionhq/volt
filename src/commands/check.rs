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

//! Check for errors

use crate::cli::{VoltCommand, VoltConfig};

use async_trait::async_trait;
use miette::Result;

/// Struct implementation for the `Check` command.
pub struct Check;

#[async_trait]
impl VoltCommand for Check {
    /// Execute the `volt Check` command
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
        Ok(())
    }
}
