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

//! Manage local node versions

use crate::{
    core::{command::Command, VERSION},
    App,
};

use std::io::Cursor;

use std::sync::Arc;

use async_trait::async_trait;
use clap::ArgMatches;
use colored::Colorize;
use miette::Result;
use node_semver::Range;
use reqwest;
use serde::de;
use serde::Deserialize;

/*
 * mod lts_status {
 *     use serde::{Deserialize, Deserializer};
 *
 *     #[derive(Deserialize, Debug, PartialEq, Eq)]
 *     #[serde(untagged)]
 *     enum LtsStatus {
 *         Nope(bool),
 *         Yes(String),
 *     }
 *
 *     impl Into<Option<String>> for LtsStatus {
 *         fn into(self) -> Option<String> {
 *             match self {
 *                 Self::Nope(_) => None,
 *                 Self::Yes(x) => Some(x),
 *             }
 *         }
 *     }
 *
 *     pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
 *     where
 *         D: Deserializer<'de>,
 *     {
 *         Ok(LtsStatus::deserialize(deserializer)?.into())
 *     }
 * }
 */

struct LtsVisitor;

/*
 * #[derive(Deserialize, Debug)]
 * struct MyJson {
 *     name: String,
 *     #[serde(deserialize_with = "from_timestamp")]
 *     timestamp: NaiveDateTime,
 * }
 */

use std::fmt;

impl<'de> de::Visitor<'de> for LtsVisitor {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string represents chrono::NaiveDateTime")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        /*
         * if let t = String::from(s) {
         *     Ok(Some(t))
         * } else {
         *     Err(de::Error::invalid_value(de::Unexpected::Str(s), &self))
         * }
         */

        /*
         * match String::from(s) {
         *     Ok(t) => Ok(Some(t)),
         *     Err(_) => Err(de::Error::invalid_value(de::Unexpected::Str(s), &self)),
         * }
         */
    }
}

fn from_timestamp<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: de::Deserializer<'de>,
{
    d.deserialize_str(LtsVisitor)
}

#[derive(Debug)]
enum Os {
    Windows,
    Macos,
    Linux,
    Unknown,
}
#[derive(Debug)]
enum Arch {
    X86,
    X64,
    Unknown,
}

fn get_arch() -> Arch {
    // "x86", "x86_64", "mips", "powerpc", "powerpc64", "arm", or "aarch64".
    if cfg!(target_arch = "x86") {
        Arch::X86
    } else if cfg!(target_arch = "x86_64") {
        Arch::X64
    } else {
        Arch::Unknown
    }
}

/// Struct implementation for the `Add` command.
#[derive(Clone)]
pub struct Node {}

impl Node {
    pub async fn download(args: &ArgMatches) -> Result<()> {
        match args.subcommand() {
            Some(("use", version)) => {
                println!("Using version {}", version.value_of("version").unwrap());
            }
            Some(("install", versions)) => {
                let v: Vec<&str> = versions.values_of("versions").unwrap().collect();
                println!("Installing version {:?}", v);
                download_node_version(v).await;
            }
            Some(("remove", versions)) => {
                let v: Vec<&str> = versions.values_of("versions").unwrap().collect();
                println!("Removing version {:?}", v);
            }
            _ => {}
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
pub struct IndexedNodeVersion {
    // pub version: Version,
    pub version: String,
    #[serde(deserialize_with = "from_timestamp")]
    pub lts: Option<String>,
    pub date: chrono::NaiveDate,
    pub files: Vec<String>,
}

// 32bit macos/linux systems cannot download a version of node >= 10.0.0
// They stopped making 32bit builds after that version
async fn download_node_version(versions: Vec<&str>) {
    let mirror = "https://nodejs.org/dist";

    // let r = reqwest::get().await;
    // let b = r.unwrap().bytes();
    // println!("{:?}", b.await);
    let mut value: Vec<IndexedNodeVersion> = reqwest::get(format!("{}/index.json", mirror))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let response = reqwest::get("https://nodejs.org/dist/node-v0.1.25.tar.gz");

    let mut file = std::fs::File::create("file.tar.gz").unwrap();
    let mut content = Cursor::new(response.await.unwrap().bytes().await.unwrap());
    std::io::copy(&mut content, &mut file).unwrap();

    /*
     * for v in value {
     *     println!("{:?}\n", v);
     * }
     */

    let os = if cfg!(target_os = "windows") {
        Os::Windows
    } else if cfg!(target_os = "macos") {
        Os::Macos
    } else if cfg!(target_os = "linux") {
        Os::Linux
    } else {
        Os::Unknown
    };

    let arch = get_arch();

    for v in versions {
        println!(
            "Installing version {} for architecture {:?} for OS {:?}",
            v, arch, os
        );
    }

    let _r: Range = "12".parse().unwrap();

    // println!("In foo func for {:?}!", os);
    // println!("Version {}", r);
}

#[async_trait]
impl Command for Node {
    /// Display a help menu for the `volt add` command.
    fn help() -> String {
        format!(
            r#"volt {}

            Add a package to your project's dependencies.
            Usage: {} {} {} {}
            Options:

            {} {} Output the version number.
            {} {} Output verbose messages on internal operations.
            {} {} Adds package as a dev dependency
            {} {} Disable progress bar."#,
            VERSION.bright_green().bold(),
            "volt".bright_green().bold(),
            "add".bright_purple(),
            "[packages]".white(),
            "[flags]".white(),
            "--version".blue(),
            "(-ver)".yellow(),
            "--verbose".blue(),
            "(-v)".yellow(),
            "--dev".blue(),
            "(-D)".yellow(),
            "--no-progress".blue(),
            "(-np)".yellow()
        )
    }

    /// Execute the `volt add` command
    ///
    /// Adds a package to dependencies for your project.
    /// ## Arguments
    /// * `app` - Instance of the command (`Arc<App>`)
    /// ## Examples
    /// ```rust
    /// // Add react to your dependencies with logging level verbose
    /// // .exec() is an async call so you need to await it
    /// Add.exec(app).await;
    /// ```
    /// ## Returns
    /// * `Result<()>`
    async fn exec(app: Arc<App>) -> Result<()> {
        println!("In Node Exec!");
        let x = app.get_packages();
        let x = x.unwrap();
        for a in x {
            println!("{:?}", a);
        }
        Ok(())
        /*
         *         // Get input packages
         *         let mut packages = app.get_packages()?;
         *
         *         // Load the existing package.json file
         *         let (mut package_file, package_file_path) = PackageJson::open("package.json")?;
         *
         *         // Construct a path to the local and global lockfile.
         *         let lockfile_path = &app.lock_file_path;
         *
         *         let global_lockfile = &app.home_dir.join(".global.lock");
         *
         *         // Load local and global lockfiles.
         *         let mut lock_file =
         *             LockFile::load(lockfile_path).unwrap_or_else(|_| LockFile::new(lockfile_path));
         *
         *         let mut global_lock_file =
         *             LockFile::load(global_lockfile).unwrap_or_else(|_| LockFile::new(global_lockfile));
         *
         *         // Create progress bar for resolving dependencies.
         *
         *         let progress_bar = ProgressBar::new(packages.len() as u64);
         *
         *         progress_bar.set_style(
         *             ProgressStyle::default_bar()
         *                 .progress_chars(PROGRESS_CHARS)
         *                 .template(&format!(
         *                     "{} [{{bar:40.green/magenta}}] {{msg:.blue}}",
         *                     "Resolving Dependencies".bright_blue()
         *                 )),
         *         );
         *
         *         // Fetch npm data including hash to fetch dependencies
         *         let data = npm::get_versions(&packages).await?;
         *
         *         // Fetch pre-flattened dependency trees from the registry
         *         let (responses, elapsed) = fetch_dep_tree(&data, &progress_bar).await?;
         *
         *         let mut dependencies: HashMap<String, VoltPackage> = HashMap::new();
         *
         *         for res in responses.iter() {
         *             let current_version = res.versions.get(&res.version).unwrap();
         *             dependencies.extend(current_version.to_owned());
         *         }
         *
         *         progress_bar.finish_with_message("[OK]".bright_green().to_string());
         *
         *         print_elapsed(dependencies.len(), elapsed);
         *
         *         let mut dependencies: Vec<_> = dependencies
         *             .iter()
         *             .map(|(_name, object)| {
         *                 let mut lock_dependencies: Vec<String> = vec![];
         *
         *                 if let Some(peer_deps) = &object.peer_dependencies {
         *                     for dep in peer_deps {
         *                         if !crate::core::utils::check_peer_dependency(&dep) {
         *                             progress_bar.println(format!(
         *                                 "{}{} {} has unmet peer dependency {}",
         *                                 " warn ".black().bright_yellow(),
         *                                 ":",
         *                                 object.name.bright_cyan(),
         *                                 &dep.bright_yellow()
         *                             ));
         *                         }
         *                     }
         *                 }
         *
         *                 if let Some(dependencies) = &object.dependencies {
         *                     for dep in dependencies {
         *                         lock_dependencies.push(dep.to_string());
         *                     }
         *                 }
         *
         *                 let object_instance = object.clone();
         *
         *                 lock_file.dependencies.insert(
         *                     DependencyID(object_instance.name, object_instance.version),
         *                     DependencyLock {
         *                         name: object.name.clone(),
         *                         version: object.version.clone(),
         *                         tarball: object.tarball.clone(),
         *                         integrity: object.integrity.clone(),
         *                         dependencies: lock_dependencies.clone(),
         *                     },
         *                 );
         *
         *                 let second_instance = object.clone();
         *
         *                 global_lock_file.dependencies.insert(
         *                     DependencyID(second_instance.name, second_instance.version.to_owned()),
         *                     DependencyLock {
         *                         name: object.name.clone(),
         *                         version: object.version.clone(),
         *                         tarball: object.tarball.clone(),
         *                         integrity: object.integrity.clone(),
         *                         dependencies: lock_dependencies,
         *                     },
         *                 );
         *
         *                 object
         *             })
         *             .collect();
         *
         *         for dep in dependencies.iter() {
         *             for package in packages.iter_mut() {
         *                 if dep.name == package.name {
         *                     package.version = Some(dep.version.clone());
         *                 }
         *             }
         *         }
         *
         *         let progress_bar = ProgressBar::new(dependencies.len() as u64);
         *
         *         progress_bar.set_style(
         *             ProgressStyle::default_bar()
         *                 .progress_chars(PROGRESS_CHARS)
         *                 .template(&format!(
         *                     "{} [{{bar:40.green/magenta}}] {{msg:.blue}}",
         *                     "Installing Packages".bright_blue()
         *                 )),
         *         );
         *
         *         dependencies.dedup();
         *
         *         dependencies
         *             .into_iter()
         *             .map(|v| install_extract_package(&app, &v))
         *             .collect::<FuturesUnordered<_>>()
         *             .inspect(|_| progress_bar.inc(1))
         *             .try_collect::<()>()
         *             .await
         *             .unwrap();
         *
         *         progress_bar.finish();
         *
         *         for package in packages {
         *             package_file.add_dependency(package);
         *         }
         *
         *         package_file.save()?;
         *         global_lock_file.save()?;
         *         lock_file.save()?;
         *
         *         Ok(())
         */
    }
}
