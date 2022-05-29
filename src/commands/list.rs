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

use async_trait::async_trait;
use clap::Parser;
use colored::Colorize;
use miette::Result;
//use node_semver::Version;

use crate::{
    cli::{VoltCommand, VoltConfig},
    core::utils::package::PackageJson,
};

#[derive(Debug, Parser)]
pub struct List {
    depth: Option<usize>,
}

// CREDIT:
// Author: sfackler
// Repo: cargo-tree (tree.rs)
// ------------------------------------------------
pub struct Symbols {
    _down: &'static str,
    tee: &'static str,
    ell: &'static str,
    right: &'static str,
}

pub static UTF8_SYMBOLS: Symbols = Symbols {
    _down: "│",
    tee: "├",
    ell: "└",
    right: "─",
};

pub static _ASCII_SYMBOLS: Symbols = Symbols {
    _down: "|",
    tee: "|",
    ell: "`",
    right: "-",
};
// ------------------------------------------------

#[async_trait]
impl VoltCommand for List {
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
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        let symbols = &UTF8_SYMBOLS;

        // grab the project's package.json file to get primary dependencies
        let (pkg_json, pkg_json_path) = match PackageJson::get() {
            Ok(p) => (Some(p.0), Some(p.1)),
            Err(_) => (None, None),
        };

        if let (Some(pkg_json), Some(pkg_json_path)) = (pkg_json, pkg_json_path) {
            // package.json exists
            let mut node_modules = pkg_json_path;

            let project_info = format!(
                "{}@{} {}",
                &pkg_json.name,
                &pkg_json.version,
                &node_modules
                    .parent()
                    .unwrap()
                    .canonicalize()
                    .unwrap()
                    .to_string_lossy()
            );
            println!("{}", project_info);

            let last = pkg_json.dependencies.clone();

            if let Some(packages) = pkg_json.dependencies {
                node_modules.pop();
                node_modules.push("node_modules");

                // unsure if this is necessary?
                if !packages.is_empty() {
                    let last = last.unwrap().into_keys().last().unwrap();

                    for package in packages {
                        if node_modules.join(&package.0).exists() {
                            let base_pkg =
                                PackageJson::get_from_dir(&node_modules.join(&package.0))
                                    .unwrap()
                                    .0;
                            // let current: Version = base_pkg.version.parse().unwrap();
                            let current = base_pkg.version;

                            let output = format!(
                                "{}@{}",
                                &base_pkg.name.truecolor(000, 255, 000),
                                &current.to_string().truecolor(000, 155, 000),
                            );

                            if last.eq(&package.0) {
                                println!("{}{} {}\n", symbols.ell, symbols.right, output);
                            } else {
                                println!("{}{} {}", symbols.tee, symbols.right, output);
                            }
                        } else {
                            let output = format!(
                                "{} {}@{}",
                                "MISSING".to_string().truecolor(255, 000, 000),
                                &package.0.truecolor(000, 255, 000),
                                &package.1.truecolor(000, 155, 000)
                            );

                            if last.eq(&package.0) {
                                println!("{}{} {}\n", symbols.ell, symbols.right, output);
                            } else {
                                println!("{}{} {}", symbols.tee, symbols.right, output);
                            }
                        }
                    }
                }
            } else {
                // package.json exists without any packages
                let output = "(No dependencies)".to_string();
                println!("{}{} {}\n", symbols.ell, symbols.right, output);
            }
        } else {
            // package.json just doesn't exist
            let output = "Missing 'package.json' file!".to_string();
            println!("{}", output);
        }

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
