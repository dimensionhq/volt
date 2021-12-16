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

// use crate::core::utils::errors;
// use crate::core::utils::package::PackageJson;
use crate::App;
use crate::Command;

use async_trait::async_trait;
// use colored::Colorize;
use miette::Result;

use std::sync::Arc;

pub struct Script {}

#[async_trait]
impl Command for Script {
    fn help() -> String {
        todo!()
    }

    /// Execute the `volt {script}` command
    ///
    /// Execute a script command (any script command specified in package.json)
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
    async fn exec(app: Arc<App>) -> Result<()> {
        
        
        Ok(())
    }
}
