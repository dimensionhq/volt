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

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use dialoguer::MultiSelect;
use regex::Regex;
use std::fs;
use std::process;
use volt_core::command::Command;

use volt_utils::app::App;

pub struct Watch {}

#[async_trait]
impl Command for Watch {
    fn help() -> String {
        todo!()
    }

    /// Execute the `volt watch` command
    ///
    /// Execute a watch command
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Scan for errors / a specific error in the code and fix it
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        println!("{}", "Scanning for errors".bright_cyan());

        // Get error to find
        if &app.args.len() > &1 {
            let _error = &app.args[1];
        }

        // Set current dir
        let current_dir = std::env::current_dir()?;

        // Set list for all JS files
        let mut files: Vec<String> = vec![];

        // Scan for all JS files
        for file in fs::read_dir(current_dir)? {
            let file_name = file?.path();
            if file_name.display().to_string().ends_with(".js") {
                // println!("{}", file_name.display().to_string());
                files.push(file_name.display().to_string());
            }
        }

        // Set list of modules which are not found
        let mut modules: Vec<String> = vec![];

        for file in files {
            let file_split: Vec<&str> = file.split(r"\").collect();
            let file_name = file_split[file_split.len() - 1];
            let output = process::Command::new("node").arg(file_name).output()?;
            let code = output.status.code().unwrap();
            if code == 1 {
                let err_message = String::from_utf8(output.stderr)?;
                // println!("error: {}", err_message);
                let re = Regex::new(r"Cannot find module '(.+)'").unwrap();
                let matches: Vec<&str> = re
                    .captures_iter(&err_message)
                    .map(|c| c.get(1).unwrap().as_str())
                    .collect();
                // println!("matches: {:?}", matches);
                for _match in matches {
                    modules.push(_match.to_string());
                }
            }
        }

        // Set args for adding packages
        let mut args: Vec<String> = vec!["add".to_string()];

        if modules.len() > 0 {
            println!("Found missing modules.\nPress {} to select the modules and {} to install the selected modules", "space".bright_cyan(), "enter".bright_cyan());
            let chosen_modules: Vec<usize> = MultiSelect::new().items(&modules).interact()?;
            for chosen_module in chosen_modules {
                let module = &modules[chosen_module];
                args.push(module.to_string());
            }
        }

        // Initialize app
        let mut app = App::initialize();

        // Set the args for the app
        app.args = args;

        // Add the modules
        volt_add::command::Add::exec(Arc::new(app)).await.unwrap();

        Ok(())
    }
}
