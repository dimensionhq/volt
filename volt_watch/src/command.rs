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

use std::fs::{read_dir, read_to_string};
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use regex::Regex;
use std::io::Write;
use std::process;
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
                if file_name.ends_with(".js") {
                    files.push(file_name);
                }
            }
        }

        if files.len() == 0 {
            println!("No JS files found");
            process::exit(0);
        }

        // let progress_bar = ProgressBar::new(files.len() as u64);

        // 5 Checks
        // -> 1: Matching Bracket Check
        // progress_bar.set_style(
        //     ProgressStyle::default_bar()
        //         .progress_chars(PROGRESS_CHARS)
        //         .template(&format!(
        //             "{} [{{bar:20.magenta/blue}}] {{pos}} / {{len}} {{msg:.yellow}}",
        //             "Scanning Code".bright_cyan()
        //         )),
        // );

        // files.insert(1, files[0].clone());
        // let mut files_iter = files.into_iter();

        // while let Some(f) = files_iter.next() {
        //     // display next 3 files to be analyzed
        //     let file_names = get_top_elements(files_iter.as_slice());
        //     let message = file_names.join(", ");
        //     progress_bar.set_message(message);

        //     let mut contents = read_to_string(f).unwrap();
        //     contents = contents.trim().to_string();
        //     let open_curly_count = contents.matches("{").count();
        //     let close_curly_count = contents.matches("}").count();
        //     let open_square_count = contents.matches("[").count();
        //     let close_square_count = contents.matches("]").count();
        //     let open_paren_count = contents.matches("(").count();
        //     let close_paren_count = contents.matches(")").count();

        //     if open_curly_count != close_curly_count {
        //         todo!()
        //     } else if open_square_count != close_square_count {
        //         todo!()
        //     } else if open_paren_count != close_paren_count {
        //         todo!()
        //     }
        //     progress_bar.inc(1);
        //     // sleep(Duration::from_millis(100));
        // }

        // progress_bar.finish_and_clear();

        // Set list of modules which are not found
        let mut module = "module".to_string();

        while module != "".to_string() {
            module = "".to_string();

            for file in &files {
                let file_split: Vec<&str> = file.split(r"\").collect();
                let file_name = file_split[file_split.len() - 1];
                // println!("file name: {}", file_name);
                let output = process::Command::new("node")
                    .arg(format!("src/{}", file_name))
                    .output()?;
                let code = output.status.code().unwrap();
                if code == 1 {
                    let err_message = String::from_utf8(output.stderr)?;
                    // println!("error: {}", err_message);
                    let re = Regex::new(r"Cannot find module '(.+)'").unwrap();
                    let matches: Vec<&str> = re
                        .captures_iter(&err_message)
                        .map(|c| c.get(1).unwrap().as_str())
                        .collect();
                    for _match in matches {
                        module = _match.to_string();
                    }
                }
            }

            // Check if module is empty
            if module == "".to_string() {
                break;
            } else {
                println!("{}{}", "Installing ".bright_cyan(), module.bright_cyan())
            }

            // Initialize app
            let mut app = App::initialize();

            // Set the args for the app
            app.args = vec!["add".to_string(), module.clone()];

            // Prompt to confirm installation
            print!("Do you want to install {} (Y/N): ", module);
            std::io::stdout()
                .flush()
                .ok()
                .expect("Could not flush stdout");
            let mut string: String = String::new();
            let _ = std::io::stdin().read_line(&mut string);
            if string.trim().to_lowercase() == "y" {
                // Add the module
                volt_add::command::Add::exec(Arc::new(app)).await.unwrap();
            }
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
