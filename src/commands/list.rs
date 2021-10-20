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

//! Compress node_modules into node_modules.pack.

use crate::{core::VERSION, App, Command};

use async_trait::async_trait;
use colored::Colorize;
use miette::Result;

use std::sync::Arc;

pub struct List;

#[async_trait]
impl Command for List {
    /// Display a help menu for the `volt list` command.
    fn help() -> String {
        format!(
            r#"volt {}
    
List dependency tree from node_modules.
Usage: {} {} {} {}
Options: 
    
  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "clone".bright_purple(),
            "[repository]".white(),
            "[flags]".white(),
            "--verbose".blue(),
            "(-v)".yellow(),
        )
    }

    /// Execute the `volt list` command
    ///
    /// List node_modules into node_modules.pack.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // List node_modules into node_modules.pack
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(_app: Arc<App>) -> Result<()> {
        // let flags = &app.flags;

        // let mut depth: u64 = 2;

        // // if flags.contains(&"--depth".to_string()) {
        // //     depth = app.args.iter().find_map(|s| s.parse().ok()).unwrap_or(2);
        // // }

        // let dirs = WalkDir::new("node_modules");

        // let dependency_paths: Vec<_> = dirs
        //     .into_iter()
        //     .filter_map(Result::ok)
        //     .filter(|entry| entry.file_type().is_dir() || entry.file_type().is_symlink())
        //     .collect();

        // if dependency_paths.len() == 1 {
        //     println!("{}", "No Dependencies Found!".bright_cyan());
        //     return Ok(());
        // } else if dependency_paths.is_empty() {
        //     println!(
        //         "{} {} {}",
        //         "Failed to find".bright_cyan(),
        //         "node_modules".bright_yellow().bold(),
        //         "folder".bright_cyan(),
        //     );
        //     return Ok(());
        // }

        // for dep in dependency_paths {
        //     let dep_path = dep.path().to_str().unwrap();
        //     let dep_path_split: Vec<&str> = dep_path.split('\\').collect();
        //     let dep_name: &str = dep_path_split[dep_path_split.len() - 1];
        //     if dep_name != "node_modules"
        //         && dep_name != "scripts"
        //         && !dep_name.starts_with("node_modules")
        //     {
        //         println!("{} {}", "-".bright_cyan(), dep_name.bright_blue().bold());
        //         let dirs = WalkDir::new(format!("node_modules/{}/node_modules", dep_name))
        //             .follow_links(true)
        //             .max_depth((depth - 1) as usize);
        //         let dependency_paths: Vec<_> = dirs
        //             .into_iter()
        //             .filter_map(Result::ok)
        //             .filter(|entry| entry.file_type().is_dir() || entry.file_type().is_symlink())
        //             .collect();

        //         for dep in dependency_paths {
        //             let dep_path = dep.path().to_str().unwrap();
        //             let dep_path_split: Vec<&str> = dep_path.split('\\').collect();
        //             let dep_name: &str = dep_path_split[dep_path_split.len() - 1];
        //             if dep_name != "node_modules"
        //                 && dep_name != "scripts"
        //                 && !dep_path.contains("lib")
        //                 && !dep_path.contains("src")
        //                 && !dep_path.contains("dist")
        //                 && !dep_path.contains("test")
        //                 && !dep_name.starts_with("node_modules")
        //             {
        //                 for _ in 0..dep_path_split.len() {
        //                     print!("  ");
        //                 }
        //                 let mut version = "".to_owned();
        //                 for file in read_dir(std::env::temp_dir().join("volt")).unwrap() {
        //                     let file_path: PathBuf = file.unwrap().path();
        //                     let file_name: &str = file_path.to_str().unwrap();
        //                     let file_split: Vec<&str> = file_name.split('\\').collect();
        //                     let name: &str = file_split[file_split.len() - 1];
        //                     if name.starts_with(dep_name) {
        //                         let file_split: Vec<&str> = name.split('@').collect();
        //                         let file_end = file_split[1];
        //                         let file_split: Vec<&str> = file_end.split(".tgz").collect();
        //                         version = file_split[0].to_owned();
        //                     }
        //                 }
        //                 let padding = 50 - (dep_path_split.len() * 2);
        //                 print!(
        //                     "{} {:<width$}",
        //                     "-".bright_purple(),
        //                     dep_name,
        //                     width = padding
        //                 );
        //                 println!("{}", version.clone().truecolor(190, 190, 190));
        //             }
        //         }
        //     }
        // }

        Ok(())
    }
}
