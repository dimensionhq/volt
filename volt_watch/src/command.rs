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

use rslint_parser::Syntax;
use std::fs::{read_dir, read_to_string};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use volt_core::command::Command;
use volt_utils::app::App;
use walkdir::WalkDir;

const PROGRESS_CHARS: &str = "=> ";
pub struct Watch {}

fn src_folder_exists() -> bool {
    for f in read_dir(".").unwrap() {
        let f = f.unwrap();
        if f.path().is_dir() && format!("{}", f.path().display()).as_str() == r".\src" {
            return true;
        }
    }
    false
}

fn get_top_elements(elements: &[String]) -> Vec<String> {
    let mut file_names: Vec<String> = vec![];
    let vec = elements[..3.min(elements.len())].to_vec();

    for item in &vec {
        let split = item.split(r"\").collect::<Vec<&str>>();
        let final_element = split.last().unwrap();
        file_names.push(final_element.to_string());
    }

    file_names
}

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
    /// //
    /// // .exec() is an async call so you need to await it
    /// Watch.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        // Set current dir
        let mut current_dir = std::env::current_dir()?;

        // Set list for all JS files
        let mut files: Vec<String> = vec![];

        // Scan for all JS files
        // Code must be in src folder
        if !current_dir.ends_with("src") && src_folder_exists() {
            current_dir = current_dir.join("src");
        }

        if current_dir.ends_with("src") {
            for entry in WalkDir::new(current_dir) {
                let entry = entry.unwrap();
                let file_name = format!("{}", entry.path().display());
                if file_name.ends_with(".js")
                    || file_name.ends_with(".ts")
                    || file_name.ends_with(".json")
                {
                    files.push(file_name);
                }
            }
        }

        if files.len() > 0 {
            let progress_bar = ProgressBar::new(files.len() as u64);

            progress_bar.set_style(
                ProgressStyle::default_bar()
                    .progress_chars(PROGRESS_CHARS)
                    .template(&format!(
                        "{} [{{bar:20.magenta/blue}}] {{pos}} / {{len}} {{msg:.yellow}}",
                        "Scanning Code".bright_cyan()
                    )),
            );

            let mut files_message_vec = files.clone();
            for f in files {
                // display next 3 files to be analyzed
                let file_names;
                file_names = get_top_elements(&files_message_vec.as_slice());

                let message = file_names.join(", ");
                progress_bar.set_message(message);

                let mut syntax = Syntax::default();

                if f.clone().ends_with(".ts") {
                    syntax = syntax.typescript();
                }

                let res = rslint_parser::parse_with_syntax(
                    read_to_string(&f).unwrap().as_str().trim(),
                    0,
                    syntax,
                );

                let errors = res.errors();

                if errors != [] {
                    progress_bar.abandon();
                    // println!("{}", &f);
                    for err in errors {
                        println!("{:?}", err);
                    }
                }

                files_message_vec.remove(0);
                progress_bar.inc(1);
            }

            progress_bar.finish_with_message("");

            // Set list of modules which are not found
            // let mut modules: Vec<String> = vec![];

            // for file in files {
            //     let file_split: Vec<&str> = file.split(r"\").collect();
            //     let file_name = file_split[file_split.len() - 1];
            //     let output = process::Command::new("node").arg(file_name).output()?;
            //     let code = output.status.code().unwrap();
            //     if code == 1 {
            //         let err_message = String::from_utf8(output.stderr)?;
            //         let re = Regex::new(r"Cannot find module '(.+)'").unwrap();
            //         let matches: Vec<&str> = re
            //             .captures_iter(&err_message)
            //             .map(|c| c.get(1).unwrap().as_str())
            //             .collect();
            //         for _match in matches {
            //             modules.push(_match.to_string());
            //         }
            //     }
            // }

            // Set args for adding packages
            // let mut args: Vec<String> = vec!["add".to_string()];
            // if modules.len() > 0 {
            //     println!("Found missing modules.\nPress {} to select the modules and {} to install the selected modules", "space".bright_cyan(), "enter".bright_cyan());
            //     let chosen_modules: Vec<usize> = MultiSelect::new().items(&modules).interact()?;
            //     for chosen_module in chosen_modules {
            //         let module = &modules[chosen_module];
            //         args.push(module.to_string());
            //     }
            // }

            // Initialize app
            // let mut app = App::initialize();

            // Set the args for the app
            // app.args = args.clone();

            // if &args.len() > &1 {
            //     // Add the modules
            //     volt_add::command::Add::exec(Arc::new(app)).await.unwrap();
            // }
        }

        Ok(())
    }
}

/*
            let split = &f.split(r"\").collect::<Vec<&str>>();
            let file_name = split.last().unwrap();
            let file_names = get_top_elements(files);
            progress_bar.set_message(file_name.to_string());

            let mut contents = read_to_string(f).unwrap();
            contents = contents.trim().to_string();
            let open_curly_count = contents.matches("{").count();
            let close_curly_count = contents.matches("}").count();
            let open_square_count = contents.matches("[").count();
            let close_square_count = contents.matches("]").count();
            let open_paren_count = contents.matches("(").count();
            let close_paren_count = contents.matches(")").count();

            if open_curly_count != close_curly_count {
                todo!()
            } else if open_square_count != close_square_count {
                todo!()
            } else if open_paren_count != close_paren_count {
                todo!()
            }

*/
