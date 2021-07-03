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

//! Display info about a package.

use std::{
    process,
    sync::{Arc, Mutex},
};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use volt_core::{command::Command, model::http_manager::get_package, VERSION};
use volt_utils::{
    app::App,
    package::{Package, PackageJson, Version},
};

pub struct Info {}

#[async_trait]
impl Command for Info {
    fn help() -> String {
        format!(
            r#"volt {}
    
Shows the information of a package 

Usage: {} {} {}

Options: 

  {} {} Output verbose messages on internal operations."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "deploy".bright_purple(),
            "[commit]".white(),
            "--verbose".blue(),
            "(-v)".yellow()
        )
    }

    /// Execute the `volt info` command
    ///
    /// Display info about a package
    /// ## Arguments
    /// * `error` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```
    /// // Display info about a package
    /// // .exec() is an async call so you need to await it
    /// Info.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        let mut name = String::new();
        if !std::env::current_dir()?.join("package.json").exists() {
            println!(
                "{}: {}\n",
                "Warning:".yellow().bold(),
                "Could not find a package.json file in the current directory"
            );
            name = volt_utils::get_basename(app.current_dir.to_str().unwrap()).to_string()
        } else {
            let package_file = PackageJson::from("package.json");
            name = package_file.name;
        }
        let package: Package = get_package(&name).await?.unwrap();
        if package.description == None {
            println!("{}", "<No description provided>".yellow().bold());
        } else {
            println!("{}\n", package.description.unwrap());
        }
        if package.keywords == None {
            println!("{}", "<No Keyword provided>".yellow().bold());
        } else {
            print!("{}: ", "Keywords".blue().bold());
            for keyword in package.keywords.unwrap().iter() {
                print!("{} ", keyword.green())
            }
            print!("\n")
        }
        print!("\n");
        let latest_version = package.dist_tags.latest;
        println!("Latest Version: v{}\n", latest_version.blue());
        let latestpackage: &Version = &package.versions[&latest_version];
        println!("dist:");
        println!("\ttarball: {}", latestpackage.dist.tarball.blue().bold());
        println!("\tshasum: {}", latestpackage.dist.shasum.blue().bold());
        println!(
            "\tintegrity: {}",
            latestpackage.dist.integrity.blue().bold()
        );
        println!(
            "\tunpackedSize: {}{}",
            (latestpackage.dist.unpacked_size / 1024)
                .to_string()
                .blue()
                .bold(),
            "kb".blue().bold()
        );

        // println!("{:#?}", latestpackage);
        println!("{}", "\nmaintainers:");
        for maintainer in latestpackage.maintainers.iter() {
            println!(
                "\t{}<{}>",
                maintainer.email,
                maintainer.name.yellow().bold()
            )
        }
        Ok(())
    }
}
