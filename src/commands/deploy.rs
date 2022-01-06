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

//! Remove a package from your direct dependencies.

use crate::cli::{VoltCommand, VoltConfig};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::sync::Arc;

/// Struct implementation for the `Deploy` command.
pub struct Deploy;

#[async_trait]
impl VoltCommand for Deploy {
    /// Execute the `volt deploy` command
    ///
    /// Removes a package from your direct dependencies.
    /// ## Arguments
    /// * `commit_msg` - Name of the commit message.
    /// ## Examples
    /// ```
    /// // .exec() is an async call so you need to await it
    /// Create.exec(app, vec![], vec!["--verbose"]).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(self, config: VoltConfig) -> Result<()> {
        // let args: Vec<String> = app.args.clone();
        // if args.is_empty() {
        //     error!("expected commit name");
        //     process::exit(1);
        // } else {
        //     let commit_msg = &args[0];
        //     env::set_current_dir(env::current_dir().unwrap()).unwrap();
        //     // println!("current dir: {:?}", env::current_dir()?);
        //     process::Command::new("git")
        //         .args(&["add", "."])
        //         .output()
        //         .expect("Failed to add");
        //     process::Command::new("git")
        //         .args(&["commit", "-m", commit_msg.as_str()])
        //         .output()
        //         .expect("Failed to commit");
        //     process::Command::new("git")
        //         .args(&["push"])
        //         .output()
        //         .expect("Failed to push");
        // }
        Ok(())
    }
}
