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

use crate::cli::{VoltCommand, VoltConfig};

use async_trait::async_trait;
use clap::Parser;
use miette::Result;

/// Display information about a package
#[derive(Debug, Parser)]
pub struct Info {}

#[async_trait]
impl VoltCommand for Info {
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
    async fn exec(self, _config: VoltConfig) -> Result<()> {
        // #[allow(unused_assignments)]
        // let mut name = String::new();

        // if !std::env::current_dir()
        //     .unwrap()
        //     .join("package.json")
        //     .exists()
        //     && app.args.len() == 1
        // {
        //     println!(
        //         "{}: {}\n",
        //         "warning".yellow().bold(),
        //         "Could not find a package.json file in the current directory"
        //     );
        //     name = utils::get_basename(app.current_dir.to_str().unwrap()).to_string();
        // }

        // let mut field: String = String::new();

        // if app.args.len() > 2 {
        //     field = String::from(&app.args[2]);
        //     name = String::from(&app.args[1]);
        // } else if app.args.len() == 2 {
        //     name = String::from(&app.args[1]);
        // }

        // let package: Package = get_package(&name).await.unwrap().unwrap();

        // if field == String::new() {
        //     let latest_version = package.dist_tags.get("latest").unwrap();
        //     println!("{}\n", format!("v{}", latest_version).bright_blue());

        //     if package.description != None {
        //         println!("{}\n", package.description.unwrap());
        //     }
        //     if package.keywords != None {
        //         print!("{}: ", "keywords".bright_blue().bold());
        //         for keyword in package.keywords.unwrap().iter() {
        //             print!("{} ", keyword.green())
        //         }
        //         print!("\n\n")
        //     }

        //     let latestpackage: &Version = &package.versions[latest_version];
        //     println!("distribution:");
        //     println!(
        //         "  tarball: {}",
        //         latestpackage.dist.tarball.bright_blue().underline()
        //     );
        //     println!("  shasum: {}", latestpackage.dist.shasum.bright_green());
        //     if latestpackage.dist.integrity != "" {
        //         println!(
        //             "  integrity: {}",
        //             latestpackage.dist.integrity.bright_blue()
        //         );
        //     }
        //     if latestpackage.dist.unpacked_size != 0 {
        //         println!(
        //             "  unpackedSize: {}{}",
        //             (latestpackage.dist.unpacked_size / 1024)
        //                 .to_string()
        //                 .bright_blue()
        //                 .bold(),
        //             "kb".bright_blue().bold()
        //         );
        //     }

        //     let dependencies = latestpackage
        //         .dependencies
        //         .keys()
        //         .cloned()
        //         .collect::<Vec<String>>();

        //     if dependencies.len() != 0 {
        //         println!("\ndependencies:");
        //         for dep in dependencies.iter() {
        //             println!("{}{}", "  - ".bright_magenta(), dep);
        //         }
        //     }

        //     println!("{}", "\nmaintainers:");
        //     for maintainer in latestpackage.maintainers.iter() {
        //         println!(
        //             "  {} {}<{}>",
        //             "-".bright_magenta(),
        //             maintainer.email,
        //             maintainer.name.yellow().bold()
        //         )
        //     }
        //     print!("\n");
        // } else {
        //     match field.as_str() {
        //         "readme" => {
        //             let text: String;

        //             if package.readme.is_some() && package.readme.as_ref().unwrap().trim() != "" {
        //                 text = package.readme.unwrap();
        //             } else {
        //                 let latest_version = package.dist_tags.get("latest").unwrap();
        //                 let current_version = package.versions.get(latest_version).unwrap();

        //                 if current_version.readme.is_some() {
        //                     text = current_version.readme.as_ref().unwrap().to_string();
        //                 } else {
        //                     text = String::new();
        //                 }
        //             }
        //             if text != String::new() {
        //                 PrettyPrinter::new()
        //                     .input_from_bytes(text.as_bytes())
        //                     .theme("Dracula")
        //                     .language("markdown")
        //                     .print()
        //                     .unwrap();

        //                 print!("\n");
        //             } else {
        //                 error!("could not find a readme for {}", name);
        //             }
        //         }
        //         "version" => {
        //             let mut table = Table::new();

        //             let versions: HashMap<String, String> = package.dist_tags;

        //             let mut labels: Vec<Cell> = vec![Cell::new("")];

        //             let mut values: Vec<Cell> =
        //                 vec![Cell::new(&name.bright_blue().to_string().as_str())];

        //             for (k, v) in versions.iter() {
        //                 labels.push(Cell::new(k.as_str()));
        //                 values.push(Cell::new(v.bright_magenta().to_string().as_str()));
        //             }

        //             table.add_row(Row::new(labels));

        //             table.add_row(Row::new(values));

        //             table.printstd();
        //             print!("\n");
        //         }
        //         "versions" => {
        //             let versions = package.versions.keys().cloned().collect::<Vec<String>>();

        //             println!("{}{}", "versions".bright_cyan(), ":".bright_magenta());
        //             for v in versions {
        //                 println!("  {} {}", "-".bright_magenta(), v.bright_green());
        //             }
        //             print!("\n");
        //         }
        //         "description" => {
        //             let description = package.description;

        //             if description.is_some() {
        //                 println!("{}", description.unwrap());
        //                 print!("\n");
        //             } else {
        //                 error!("could not find a description for {}", name);
        //             }
        //         }
        //         "name" => {
        //             println!("{}", name);
        //             print!("\n");
        //         }
        //         "maintainers" => {
        //             for maintainer in package.maintainers.iter() {
        //                 println!(
        //                     "{} <{}>",
        //                     maintainer.name.bright_green().bold(),
        //                     maintainer.email.bright_magenta(),
        //                 )
        //             }
        //             print!("\n");
        //         }
        //         "time" => {
        //             let times = package.time;

        //             for (version, time) in times {
        //                 println!(
        //                     "{} {} {}",
        //                     version.bright_cyan(),
        //                     ":".bright_magenta(),
        //                     time.bright_black()
        //                 );
        //             }

        //             print!("\n");
        //         }
        //         "repository" => {
        //             let repo = package.repository;

        //             if repo.is_some() {
        //                 let data = repo.unwrap();

        //                 println!("{}: {}", "provider".bright_yellow(), data.type_field);
        //                 println!("{}: {}", "url".bright_purple(), data.url.underline());
        //                 print!("\n");
        //             } else {
        //                 error!("could not find a repository for {}", name);
        //             }
        //         }
        //         "homepage" => {
        //             let page = package.homepage;

        //             if page.is_some() {
        //                 let data = page.unwrap();

        //                 println!("{}", data);
        //                 print!("\n");
        //             } else {
        //                 error!("could not find a homepage for {}", name);
        //             }
        //         }
        //         "keywords" => {
        //             let keywords = package.keywords;
        //             println!("{}{}", "keywords".bright_cyan(), ":".bright_cyan());
        //             if keywords.is_some() {
        //                 let keywords = keywords.unwrap();

        //                 for kw in keywords {
        //                     println!("{} {}", "-".bright_magenta(), kw);
        //                 }
        //                 print!("\n");
        //             } else {
        //                 error!("could not find keywords for {}", name);
        //             }
        //         }
        //         "users" => {}
        //         &_ => {}
        //     }
        // }

        Ok(())
    }
}
