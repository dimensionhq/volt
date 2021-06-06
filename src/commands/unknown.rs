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

// Std Imports
use std::sync::Arc;

// Library Imports
use anyhow::Result;
use async_trait::async_trait;

// Crate Level Imports
use crate::{classes::package::PackageJson, utils::App};

// Super Imports
use super::Command;

pub struct Unknown {}

#[async_trait]
impl Command for Unknown {
    fn help() -> String {
        todo!()
    }

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
    async fn exec(app: Arc<App>) -> Result<()> {
        let package_json = PackageJson::from("package.json");

        let args = app.args.clone();
        let command: &str = args[0].as_str();

        if package_json.scripts.contains_key(command) {
            let script = package_json.scripts.get(command).unwrap();
            let mut split: Vec<&str> = script.split_ascii_whitespace().into_iter().collect();
            let bin_cmd = format!("{}.cmd", split[0]);

            split[0] = bin_cmd.as_str();

            let exec = format!("node_modules\\scripts\\{}", split.join(" "));

            std::process::Command::new("cmd.exe")
                .arg("/C")
                .arg(exec)
                .spawn()
                .unwrap();
        } else {
        }

        Ok(())
    }
}
